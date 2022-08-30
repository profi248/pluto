#[cfg(test)]
#[path = "blob_index_tests.rs"]
mod blob_index_tests;

use std::cmp::max;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{ Read, Write };

use aes_gcm::{ AeadInPlace, Aes256Gcm, Key, Nonce, KeyInit };
use bincode::Options;

use pluto_network::key::Keys;
use crate::pack::{ BlobHash, PackfileId };
use super::PackfileError;

const MAX_FILE_ENTRIES: usize = 50_000;

const NONCE_SIZE: usize = 12;
const KEY_DERIVATION_CONSTANT: &[u8] = b"index";

type Entry = Vec<(BlobHash, PackfileId)>;

/// Index is a set of files containing mappings of blob => packfile.
/// It is useful for quickly finding the packfile to fetch if we want a particular blob.
/// The source of truth for data stored in packfiles are their headers, index
/// is just a tool for making seeking easier. Index can always be reconstructed
/// from packfile headers, but is primarily written along with newly created packfiles.
///
/// On disk, index consists of many files in a folder, using a sequential numbering system.
/// To load the index in memory, all files are combined into one long list. The reason for splitting
/// index into individual files is to make additions easy, and to prevent files from growing too large.
/// The index files are also encrypted before saving, with a key derived from master key and constant,
/// and the index ID as nonce. Index capacity is capped to 50 000 entries, making the largest file
/// size slightly larger than 2 MiB.
///
/// TODO improve space efficiency of index (if needed):
/// To save space, we'll only store the minimum amount required to uniquely find a blob.
/// At first only the initial 3 bytes will be saved, if a collision is found, we will store as
/// many bytes as necessary. When reading index, the longest matching blob hash will be
/// treated as the correct entry. Best case scenario hash will only take up 4 bytes
/// (3 bytes of hash + 1 byte of length) instead of 32 bytes. Size per index entry is
/// 4-33 + 12 bytes, or 20-45 in total. For 2 500 000 stored blobs (that is about 5 TB of data
/// or 2 500 000 individual files), total index size would range from about 50 to 112 megabytes.
pub(crate) struct BlobIndex {
    /// Path to the index folder.
    output_path: String,
    /// Index entries loaded from disk.
    items: Entry,
    /// Index entries waiting to be written to disk.
    items_buf: Entry,
    /// All blob hashes that have been queued for writing or have already been written.
    blobs_queued: HashSet<BlobHash>,
    /// Numeric ID of the last written index file.
    last_file_num: u32,
    /// Keys used to encrypt index files.
    keys: Keys,
    /// Keeps track if index has been successfully flushed to disk.
    dirty: bool,
}

pub(crate) struct IndexPackfileHandle {
    /// Blobs contained in the a currently constructed packfile.
    blobs: Vec<BlobHash>
}

impl BlobIndex {
    pub fn new(output_path: String, keys: Keys) -> Result<Self, PackfileError> {
        fs::create_dir_all(output_path.clone())?;
        let index_files = fs::read_dir(output_path.clone())?;

        let mut max_num = 0;
        for entry in index_files {
            // Ignore files that don't match our pattern.
            max_num = max((entry?.file_name().into_string()?).parse::<u32>().unwrap_or(0), max_num);
        }

        Ok(Self {
            output_path,
            items: Default::default(),
            items_buf: Default::default(),
            blobs_queued: Default::default(),
            last_file_num: max_num,
            keys,
            dirty: false
        })
    }

    pub fn begin_packfile(&mut self) -> IndexPackfileHandle {
        IndexPackfileHandle {
            blobs: Default::default()
        }
    }

    pub fn add_to_packfile(&mut self, handle: &mut IndexPackfileHandle, blob_hash: BlobHash) -> Result<(), PackfileError> {
        handle.blobs.push(blob_hash);

        if !self.blobs_queued.insert(blob_hash) {
            return Err(PackfileError::DuplicateBlob);
        }

        Ok(())
    }

    pub fn finalize_packfile(&mut self, handle: &IndexPackfileHandle, packfile_hash: PackfileId) -> Result<(), PackfileError> {
        for blob_hash in handle.blobs.iter() {
            self.push(blob_hash, &packfile_hash)?;
        }

        Ok(())
    }

    pub fn is_blob_duplicate(&mut self, blob_hash: &BlobHash) -> Result<bool, PackfileError> {
        if self.blobs_queued.contains(blob_hash) { return Ok(true); }
        if self.find_packfile(blob_hash)?.is_some() { return Ok(true); }

        Ok(false)
    }

    pub fn find_packfile(&mut self, blob_hash: &BlobHash) -> Result<Option<PackfileId>, PackfileError> {
        if self.items.len() == 0 { self.load()? }

        match self.items.binary_search_by_key(&blob_hash, |(a, _)| &a) {
            Ok(entry_idx) => { Ok(Some(self.items[entry_idx].1)) }
            Err(_) => { Ok(None) }
        }
    }

    fn push(&mut self, blob_hash: &BlobHash, packfile_hash: &PackfileId) -> Result<(), PackfileError> {
        self.items_buf.push((*blob_hash, *packfile_hash));
        self.dirty = true;

        if self.items_buf.len() >= MAX_FILE_ENTRIES {
            self.flush()?
        }

        Ok(())
    }

    fn load(&mut self) -> Result<(), PackfileError> {
        let index_files = fs::read_dir(&self.output_path)?;

        for entry in index_files {
            let entry = entry?;
            // Ignore files that don't match our pattern.
            let file_num = (entry.file_name().into_string()?).parse::<u32>();
            if file_num.is_ok() {
                let mut file = File::open(entry.path())?;
                let mut buf: Vec<u8> = Default::default();
                file.read_to_end(&mut buf)?;

                let key = self.keys.derive_symmetric_key(KEY_DERIVATION_CONSTANT);
                let cipher = Aes256Gcm::new(&key.into());
                let nonce_bytes = self.counter_to_nonce(file_num.unwrap());
                let nonce = Nonce::from_slice(&nonce_bytes);

                cipher.decrypt_in_place(nonce, b"", &mut buf)?;

                let mut items: Entry = bincode::options().with_varint_encoding().deserialize(&buf)?;
                self.items.append(&mut items);
            }
        }

        // Sort all the entries so we're able to use binary search.
        self.items.sort_unstable_by_key(|&(a, _)| a);

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), PackfileError> {
        let mut buf = bincode::options().with_varint_encoding()
            .serialize(&self.items_buf)?;
        let new_file_num = self.last_file_num.checked_add(1)
            .expect("bug: index file counter overflow");

        // Derive a key for index and let nonce be the index file number.
        let key = self.keys.derive_symmetric_key(KEY_DERIVATION_CONSTANT);
        let cipher = Aes256Gcm::new(&key.into());
        let nonce_bytes = self.counter_to_nonce(new_file_num);
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher.encrypt_in_place(nonce, b"", &mut buf)?;

        let file_path = format!("{}/{:0>10}", self.output_path, new_file_num);
        let mut file = File::create(file_path.clone())?;
        file.write_all(&buf)?;

        self.last_file_num = new_file_num;
        self.items_buf.clear();
        self.dirty = false;

        Ok(())
    }

    fn counter_to_nonce(&self, file_number: u32) -> [u8; NONCE_SIZE] {
        let mut nonce_bytes = [0; NONCE_SIZE];
        nonce_bytes[0..4].copy_from_slice(&file_number.to_le_bytes());

        nonce_bytes
    }
}

impl Drop for BlobIndex {
    fn drop(&mut self) {
        if self.dirty {
            panic!("Index was dropped while dirty, without calling flush()");
        }
    }
}
