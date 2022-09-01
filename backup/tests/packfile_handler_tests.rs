use rand::{ Rng, RngCore, SeedableRng };
use rand_chacha::ChaCha8Rng;
use sha2::{ Sha256, Digest };
use tokio::fs;
use tokio_stream::{ StreamExt, wrappers::ReadDirStream };

use pluto_backup::pack::{
    packfile_handler::{ PackfileHandler, BLOB_MAX_UNCOMPRESSED_SIZE },
    BlobKind, Blob
};
use pluto_network::key::Keys;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_packer() {
    let dir = format!("{}/pack", std::env::temp_dir().to_str().unwrap());
    fs::remove_dir_all(dir.clone()).await.ok();

    let keys = Keys::from_entropy([0; 32]);
    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).await.unwrap();

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

    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).await.unwrap();
    assert_eq!(blob1, packer.get_blob(blob1.hash).await.unwrap().unwrap());
    assert_eq!(blob2, packer.get_blob(blob2.hash).await.unwrap().unwrap());
    assert_eq!(None, packer.get_blob([2; 32]).await.unwrap());

    fs::remove_dir_all(dir.clone()).await.ok();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_packer_deduplication() {
    let dir = format!("{}/pack2", std::env::temp_dir().to_str().unwrap());
    fs::remove_dir_all(dir.clone()).await.ok();

    let keys = Keys::from_entropy([0; 32]);
    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).await.unwrap();
    let mut rng = ChaCha8Rng::seed_from_u64(0);

    let mut data: Vec<u8> = vec![0; 1_000_000];
    rng.fill_bytes(&mut data);

    let blob = Blob {
        hash: [0; 32],
        kind: BlobKind::FileChunk,
        data
    };

    for _ in 1..1000 {
        packer.add_blob(blob.clone()).await.expect("Failed to add blob");
    }

    packer.flush().await.expect("Failed to finish");

    let mut total_size = 0;

    let mut iter = ReadDirStream::new(fs::read_dir(dir.clone()).await.unwrap());
    while let Some(item) = iter.next().await {
        total_size += item.unwrap().metadata().await.unwrap().len();
    }

    assert!(total_size < 5_000_000);

    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).await.unwrap();

    for _ in 1..1000 {
        packer.add_blob(blob.clone()).await.expect("Failed to add blob");
    }

    let mut total_size = 0;

    let mut iter = ReadDirStream::new(fs::read_dir(dir.clone()).await.unwrap());
    while let Some(item) = iter.next().await {
        total_size += item.unwrap().metadata().await.unwrap().len();
    }

    packer.flush().await.expect("Failed to finish");
    assert!(total_size < 5_000_000);

    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).await.unwrap();
    assert_eq!(blob, packer.get_blob(blob.clone().hash).await.unwrap().unwrap());

    fs::remove_dir_all(dir.clone()).await.ok();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_packer_rand() {
    let dir = format!("{}/pack3", std::env::temp_dir().to_str().unwrap());
    fs::remove_dir_all(dir.clone()).await.ok();

    let keys = Keys::from_entropy([0; 32]);
    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).await.unwrap();
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
    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).await.unwrap();

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
    let mut packer = PackfileHandler::new(dir.clone(), keys.clone()).await.unwrap();

    for blob in &blobs {
        assert_eq!(*blob, packer.get_blob(blob.hash).await.unwrap().unwrap());
    }

    fs::remove_dir_all(dir.clone()).await.ok();
}
