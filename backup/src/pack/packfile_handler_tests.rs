use rand::{ Rng, RngCore, SeedableRng };
use rand_chacha::ChaCha8Rng;
use sha2::{ Sha256, Digest };

use crate::pack::BlobKind;
use super::*;

#[tokio::test]
async fn test_packer() {
    let dir = format!("{}/pack", std::env::temp_dir().to_str().unwrap());
    fs::remove_dir_all(dir.clone()).ok();

    let keys = Keys::from_entropy([0; 32]);
    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).unwrap();

    let blob1 = Blob {
        hash: [0; 32],
        kind: BlobKind::FileChunk,
        data: vec![1, 2, 3]
    };

    packer.add_blob(blob1.clone()).await.expect("Failed to add blob");

    let blob2 = Blob {
        hash: [1; 32],
        kind: BlobKind::FileChunk,
        data: vec![4, 5, 6]
    };
    packer.add_blob(blob2.clone()).await.expect("Failed to add blob");

    packer.flush().await.expect("Failed to finish");

    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).unwrap();
    assert_eq!(blob1, packer.get_blob(blob1.hash).await.unwrap().unwrap());
    assert_eq!(blob2, packer.get_blob(blob2.hash).await.unwrap().unwrap());
    assert_eq!(None, packer.get_blob([2; 32]).await.unwrap());

    fs::remove_dir_all(dir.clone()).ok();
}

#[tokio::test]
async fn test_packer_deduplication() {
    let dir = format!("{}/pack2", std::env::temp_dir().to_str().unwrap());
    fs::remove_dir_all(dir.clone()).ok();

    let keys = Keys::from_entropy([0; 32]);
    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).unwrap();
    let mut rng = ChaCha8Rng::seed_from_u64(0);

    let mut data: Vec<u8> = vec![0; 1_000_000];
    rng.fill_bytes(&mut data);

    let blob = Blob {
        hash: [0; 32],
        kind: BlobKind::FileChunk,
        data
    };

    for i in 1..1000 {
        packer.add_blob(blob.clone()).await.expect("Failed to add blob");
    }

    packer.flush().await.expect("Failed to finish");

    let mut total_size = 0;
    for item in fs::read_dir(dir.clone()).unwrap() {
        total_size += item.unwrap().metadata().unwrap().len();
    }

    assert!(total_size < 5_000_000);

    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).unwrap();

    for i in 1..1000 {
        packer.add_blob(blob.clone()).await.expect("Failed to add blob");
    }

    let mut total_size = 0;
    for item in fs::read_dir(dir.clone()).unwrap() {
        total_size += item.unwrap().metadata().unwrap().len();
    }

    packer.flush().await.expect("Failed to finish");
    assert!(total_size < 5_000_000);

    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).unwrap();
    assert_eq!(blob, packer.get_blob(blob.clone().hash).await.unwrap().unwrap());

    fs::remove_dir_all(dir.clone()).ok();
}

#[tokio::test]
async fn test_packer_rand() {
    let dir = format!("{}/pack3", std::env::temp_dir().to_str().unwrap());
    fs::remove_dir_all(dir.clone()).ok();

    let keys = Keys::from_entropy([0; 32]);
    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).unwrap();
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let mut blobs = vec![];

    for _ in 0..50 {
        let size = rng.gen_range(1..BLOB_MAX_UNCOMPRESSED_SIZE);
        let mut data: Vec<u8> = vec![0; size];
        rng.fill_bytes(&mut data[0..size]);
        let hash = Sha256::digest(&data);

        let blob = Blob {
            hash: hash.into(),
            kind: BlobKind::FileChunk,
            data
        };

        blobs.push(blob.clone());
        packer.add_blob(blob).await.unwrap();
    }

    packer.flush().await.unwrap();
    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).unwrap();

    for blob in &blobs {
        assert_eq!(*blob, packer.get_blob(blob.hash).await.unwrap().unwrap());
    }

    for _ in 0..2500 {
        let size = rng.gen_range(10..10_000);
        let mut data: Vec<u8> = vec![0; size];
        rng.fill_bytes(&mut data[0..size]);
        let hash = Sha256::digest(&data);

        let blob = Blob {
            hash: hash.into(),
            kind: BlobKind::FileChunk,
            data
        };

        blobs.push(blob.clone());
        packer.add_blob(blob).await.unwrap();
    }

    packer.flush().await.unwrap();
    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).unwrap();

    for blob in &blobs {
        assert_eq!(*blob, packer.get_blob(blob.hash).await.unwrap().unwrap());
    }

    fs::remove_dir_all(dir.clone()).ok();
}

#[test]
fn validate_size_constraints() {
    let entry = PackfileBlob {
        hash: [0; 32],
        kind: BlobKind::FileChunk,
        compression: CompressionKind::Zstd,
        offset: 0,
        length: 0
    };

    let entry_len = bincode::options().with_varint_encoding()
        .serialize(&entry).unwrap().len();

    // worst case scenario with maximum amount of blobs, target size reached and
    // a maximum size blob added over the target size
    assert!(PACKFILE_TARGET_SIZE + BLOB_MAX_UNCOMPRESSED_SIZE + (entry_len * PACKFILE_MAX_BLOBS) + NONCE_SIZE
        <= PACKFILE_MAX_SIZE);
}
