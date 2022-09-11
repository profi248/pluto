pub mod packfile_handler;
pub mod blob_index;

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
pub struct Blob {
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


#[derive(Debug, thiserror::Error)]
pub enum PackfileError {
    #[error("Invalid packfile header size")]
    InvalidHeaderSize,
    #[error("Packfile too large")]
    PackfileTooLarge,
    #[error("Blob found in index, but not in packfile. Index might be out of date")]
    IndexHeaderMismatch,
    #[error("Blob too large")]
    BlobTooLarge,
    #[error("Duplicate blob in packfile index")]
    DuplicateBlob,
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("Data decryption/encryption error")]
    CryptoError(#[from] aes_gcm::Error),
    #[error("{0}")]
    SerializationError(#[from] bincode::Error),
    #[error("{0}")]
    GetrandomError(#[from] getrandom::Error),
    #[error("Invalid Unicode string: {0:?}")]
    InvalidString(OsString),
}
