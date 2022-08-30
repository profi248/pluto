mod packfile_handler;
mod blob_index;

use std::ffi::OsString;
use serde::{ Serialize, Deserialize };

type BlobHash = [u8; 32];
type PackfileId = [u8; 12];

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
pub enum BlobKind { FileChunk, Tree }
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
pub enum CompressionKind { None, Zstd }

pub enum TreeKind { File, Dir }

#[derive(Serialize, Deserialize)]
struct PackfileBlob {
    hash: BlobHash,
    kind: BlobKind,
    compression: CompressionKind,
    length: u64,
    offset: u64
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Blob {
    pub hash: BlobHash,
    pub kind: BlobKind,
    pub data: Vec<u8>
}

struct Snapshot {
    id: u64,
    timestamp: u64,
    tree: BlobHash
}

struct TreeMetadata {
    size: Option<u64>,
    mtime: Option<u64>,
    ctime: Option<u64>
}

struct Tree {
    kind: TreeKind,
    name: String,
    metadata: TreeMetadata,
    children: Vec<BlobHash>,
    next_sibling: Option<BlobHash>
}


#[derive(Debug)]
pub enum PackfileError {
    InvalidHeaderSize,
    PackfileTooLarge,
    IndexHeaderMismatch,
    BlobTooLarge,
    DuplicateBlob,
    IoError(std::io::Error),
    CryptoError(aes_gcm::Error),
    SerializationError(bincode::Error),
    GetrandomError(getrandom::Error),
    StringError(OsString)
}

impl From<std::io::Error> for PackfileError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<aes_gcm::Error> for PackfileError {
    fn from(e: aes_gcm::Error) -> Self {
        Self::CryptoError(e)
    }
}

impl From<bincode::Error> for PackfileError {
    fn from(e: bincode::Error) -> Self {
        Self::SerializationError(e)
    }
}

impl From<getrandom::Error> for PackfileError {
    fn from(e: getrandom::Error) -> Self {
        Self::GetrandomError(e)
    }
}

impl From<OsString> for PackfileError {
    fn from(e: OsString) -> Self {
        Self::StringError(e)
    }
}
