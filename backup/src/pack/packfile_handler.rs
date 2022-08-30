#[cfg(test)]
#[path = "packfile_handler_tests.rs"]
mod packfile_handler_tests;

use std::fs::{ self, File, OpenOptions };
use std::io::{ Read, Seek, Write };
use std::collections::VecDeque;

use zstd::bulk::{ Compressor, Decompressor };
use aes_gcm::{ AeadInPlace, Aes256Gcm, Key, Nonce, KeyInit };
use bincode::Options;

use pluto_network::key::Keys;
use crate::pack::{ Blob, PackfileError, PackfileBlob, CompressionKind, BlobHash, PackfileId };
use crate::pack::blob_index::BlobIndex;

/// Maximum size of blob data that's allowed in a packfile.
const BLOB_MAX_UNCOMPRESSED_SIZE: usize = 3 * 1024 * 1024; // 3 MiB
/// Total blob size, after which it's attempted to write the packfile to disk.
const PACKFILE_TARGET_SIZE: usize = 2 * 1024 * 1024; // 2 MiB
/// Maximum possible size of a packfile.
const PACKFILE_MAX_SIZE: usize = 12 * 1024 * 1024; // 12 MiB
/// Maximum number of blobs that can be stored in a packfile.
const PACKFILE_MAX_BLOBS: usize = 100_000;

const ZSTD_COMPRESSION_LEVEL: i32 = 5;

const NONCE_SIZE: usize = 12;
const KEY_DERIVATION_CONSTANT_HEADER: &[u8] = b"header";

const PACKFILE_FOLDER: &str = "pack";
const INDEX_FOLDER: &str = "index";

/// A class used for writing and reading packfiles, a file format used for storing blobs efficiently
/// and securely. Packfiles can contain one or more blobs, and are useful from preventing the
/// existence of many small loose files, and instead pack those small files together so all
/// packfiles are around the same size. PackfileHandler is also responsible for deduplicating blobs
/// so identical data is only stored once. Along with the packfiles, an index is also stored in
/// the output folder to allow for quick seeking.
///
/// When creating a backup, PackfileHandler takes in blobs, compresses and encrypts them and saves
/// them to packfiles. A reverse operation is done when restoring a backup. Packfiles are stored in
/// a folder named "pack" in the output folder, and the index is stored in a folder named "index"
/// in the output folder.
///
/// The format of a packfile is as follows:
/// - header length [8 bytes]
/// - encrypted header (encoded with bincode)
///     - (for each blob):
///         - blob hash (of unencrypted, uncompressed data)
///         - blob kind (file or directory)
///         - blob compression type (currently only zstd)
///         - blob data length (after encryption)
///         - blob data offset (from start of blob section, including nonce)
/// - individually encrypted blob data
///     - (for each blob):
///         - nonce [12 bytes]
///         - actual encrypted blob data
///
/// The header is encrypted with a key derived from the master key and a constant. Packfile ID is
/// random and used as a nonce for encrypting the header. Each blob is encrypted with a key derived
/// from the master key and the specific blob hash, nonce is random and stored along with the
/// encrypted data. All encryption/authentication is done using AES-256-GCM. Header length is
/// currently not encrypted, so it's possible to estimate the number of blobs stored in a file,
/// but I don't think it's a big problem now.
pub(crate) struct PackfileHandler {
    /// The path to the output folder.
    output_path: String,
    /// Keys used to derive encryption keys.
    keys: Keys,
    /// Index struct managing blob => packfile mapping.
    index: BlobIndex,
    /// Blobs in queue to be written to packfile.
    blobs: VecDeque<Blob>,
    /// Keeps track if data has been successfully flushed to disk.
    dirty: bool
}

impl PackfileHandler {
    pub fn new(output_path: String, keys: Keys) -> Result<Self, PackfileError> {
        let packfile_path = format!("{}/{}", output_path, PACKFILE_FOLDER);
        let index_path = format!("{}/{}", output_path, INDEX_FOLDER);

        Ok(Self {
            output_path: packfile_path,
            keys: keys.clone(),
            index: BlobIndex::new(index_path, keys)?,
            blobs: VecDeque::new(),
            dirty: false
        })
    }

    pub async fn add_blob(&mut self, blob: Blob) -> Result<(), PackfileError> {
        if blob.data.len() > BLOB_MAX_UNCOMPRESSED_SIZE { return Err(PackfileError::BlobTooLarge) }
        self.blobs.push_back(blob);
        self.dirty = true;
        self.trigger_write_if_desired().await?;
        Ok(())
    }

    pub async fn get_blob(&mut self, blob_hash: BlobHash) -> Result<Option<Blob>, PackfileError> {
        if let Some(packfile_id) = self.index.find_packfile(&blob_hash)? {
            let path = self.get_packfile_path(packfile_id, false)?;
            let mut packfile = File::open(path)?;
            let packfile_size = packfile.metadata()?.len();
            if packfile_size > PACKFILE_MAX_SIZE as u64 {
                return Err(PackfileError::PackfileTooLarge)
            }

            let mut header_size_bytes: [u8; core::mem::size_of::<u64>()] = Default::default();
            packfile.read_exact(&mut header_size_bytes)?;
            let header_size = u64::from_le_bytes(header_size_bytes);

            if header_size > packfile_size || header_size == 0 {
                return Err(PackfileError::InvalidHeaderSize);
            }

            let mut header_buf = vec![0; header_size as usize];
            packfile.read_exact(&mut header_buf)?;

            let key = self.keys.derive_symmetric_key(KEY_DERIVATION_CONSTANT_HEADER);
            let cipher = Aes256Gcm::new(&key.into());
            cipher.decrypt_in_place(Nonce::from_slice(&packfile_id), b"", &mut header_buf)?;

            let header: Vec<PackfileBlob> = bincode::options().with_varint_encoding()
                .deserialize(&header_buf)?;

            for blob_metadata in header {
                if blob_metadata.hash == blob_hash {
                    let mut blob_nonce = [0; NONCE_SIZE];
                    let mut blob_buf = vec![0; blob_metadata.length as usize];
                    packfile.seek(std::io::SeekFrom::Current(blob_metadata.offset as i64))?;

                    packfile.read_exact(&mut blob_nonce)?;
                    packfile.read_exact(&mut blob_buf)?;

                    let key = self.keys.derive_symmetric_key(&blob_metadata.hash);
                    let cipher = Aes256Gcm::new(&key.into());
                    cipher.decrypt_in_place(Nonce::from_slice(&blob_nonce), b"", &mut blob_buf)?;

                    let mut decompressor = Decompressor::new()?;
                    decompressor.include_magicbytes(false)?;
                    let blob_data = decompressor.decompress(&mut blob_buf, BLOB_MAX_UNCOMPRESSED_SIZE)?;

                    return Ok(Some(Blob {
                        hash: blob_metadata.hash,
                        kind: blob_metadata.kind,
                        data: blob_data
                    }));
                }
            }

            return Err(PackfileError::IndexHeaderMismatch);
        } else {
            // todo handle index not having the blob better
            return Ok(None);
        }
    }

    pub async fn flush(&mut self) -> Result<(), PackfileError> {
        self.write_packfiles().await?;
        self.index.flush()?;
        self.dirty = false;

        Ok(())
    }

    async fn trigger_write_if_desired(&mut self) -> Result<bool, PackfileError> {
        let mut candidates_size: usize = 0;
        let mut candidates_cnt: usize = 0;
        for blob in &self.blobs {
            if !self.index.is_blob_duplicate(&blob.hash)? {
                candidates_size += blob.data.len();
                candidates_cnt += 1;
            }

            if candidates_size >= PACKFILE_TARGET_SIZE || candidates_cnt >= PACKFILE_MAX_BLOBS {
                return self.write_packfiles().await.map(|_| true);
            }
        }

        Ok(false)
    }

    async fn write_packfiles(&mut self) -> Result<(), PackfileError> {
        while !self.blobs.is_empty() {
            let mut packfile_index = self.index.begin_packfile();
            let mut data: Vec<u8> = Vec::new();
            let mut header: Vec<PackfileBlob> = Vec::new();
            let mut blob_count: usize = 0;
            let mut bytes_written: usize = 0;

            let mut compressor = Compressor::new(ZSTD_COMPRESSION_LEVEL)?;
            compressor.include_checksum(false)?;
            compressor.include_contentsize(false)?;
            compressor.include_magicbytes(false)?;

            while let Some(blob) = &self.blobs.pop_front() {
                // Deduplication - if the blob is already saved in this or other existing
                // packfiles, skip it.
                if self.index.is_blob_duplicate(&blob.hash)? { continue; }

                // Derive a new key for each for each blob based on the (unencrypted) hash,
                // to ensure that we have a unique nonce/key combo.
                let key = self.keys.derive_symmetric_key(&blob.hash);
                let cipher = Aes256Gcm::new(&key.into());

                let mut blob_data = compressor.compress(&blob.data)?;

                // Generate a random nonce for each blob.
                let mut nonce_bytes: [u8; NONCE_SIZE] = Default::default();
                getrandom::getrandom(&mut nonce_bytes)?;
                let nonce = Nonce::from_slice(&nonce_bytes);

                cipher.encrypt_in_place(nonce, b"", &mut blob_data)?;

                // Add blob to header.
                header.push(PackfileBlob {
                    hash: blob.hash.clone(),
                    kind: blob.kind,
                    compression: CompressionKind::Zstd,
                    offset: bytes_written as u64,
                    length: blob_data.len() as u64
                });

                bytes_written += blob_data.len() + NONCE_SIZE;

                // Write blob to packfile buffer, as nonce[NONCE_SIZE] || encrypted_data[length].
                data.append(&mut nonce_bytes.to_vec());
                data.append(&mut blob_data);

                self.index.add_to_packfile(&mut packfile_index, blob.hash)?;

                blob_count += 1;

                if bytes_written >= PACKFILE_TARGET_SIZE || blob_count >= PACKFILE_MAX_BLOBS {
                    break;
                }
            }

            // If no blobs were added to the packfile because of deduplication, skip writing it.
            if blob_count == 0 { continue; }

            // Generate a random packfile ID that will be used as a filename and a nonce for the header.
            let mut packfile_id: PackfileId = Default::default();
            getrandom::getrandom(&mut packfile_id)?;

            // Derive a key for headers based on a constant.
            let key = self.keys.derive_symmetric_key(KEY_DERIVATION_CONSTANT_HEADER);
            let cipher = Aes256Gcm::new(&key.into());

            let mut header: Vec<u8> = bincode::options().with_varint_encoding().serialize(&header)?;
            cipher.encrypt_in_place(&Nonce::from_slice(&packfile_id), b"", &mut header)?;

            let mut buffer: Vec<u8> = Vec::with_capacity(
                core::mem::size_of::<u64>() + header.len() + bytes_written
            );

            // Create a packfile buffer with the following structure:
            // header_length[sizeof u64] || encrypted_header[header_length] || data.
            buffer.append(&mut (header.len() as u64).to_le_bytes().to_vec());
            buffer.append(&mut header);
            buffer.append(&mut data);

            assert!(buffer.len() <= PACKFILE_MAX_SIZE,
                    "bug: violated packfile size limit ({} B)", buffer.len());

            let file_path = self.get_packfile_path(packfile_id, true)?;

            // Ensure that we are not overwriting an existing packfile by chance.
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(file_path)?;

            file.write_all(&buffer)?;

            self.index.finalize_packfile(&mut packfile_index, packfile_id)?;
            debug!("wrote packfile {} of size {}", hex::encode(packfile_id), buffer.len());
        }

        Ok(())
    }

    fn get_packfile_path(&mut self, packfile_hash: PackfileId, create_folders: bool) -> Result<String, PackfileError> {
        let packfile_hash_hex = hex::encode(packfile_hash);

        // Split packfiles into directories based on the first two hex characters of the hash,
        // to avoid having too many files in the same directory.
        let directory = format!("{}/{}", self.output_path, &packfile_hash_hex[..2]);
        let file_path = format!("{}/{}", directory, packfile_hash_hex);

        if create_folders { fs::create_dir_all(directory)? };

        Ok(file_path)
    }
}

impl Drop for PackfileHandler {
    fn drop(&mut self) {
        if self.dirty {
            panic!("Packer was dropped while dirty, without calling flush()");
        }
    }
}
