use tokio::fs;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{ RngCore, SeedableRng };
use rand::seq::SliceRandom;

use pluto_network::key::Keys;
use pluto_backup::pack::blob_index::BlobIndex;

#[tokio::test]
async fn test_index() {
    let dir = format!("{}/idx", std::env::temp_dir().to_str().unwrap());
    let keys = Keys::from_entropy([0; 32]);

    fs::remove_dir_all(dir.clone()).await.ok();

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();

    let mut packfile_handle = idx.begin_packfile();

    for i in 0..=100 {
        let blob_hash = [i; 32];
        idx.add_to_packfile(&mut packfile_handle, blob_hash).unwrap();
    }

    assert_eq!(true, idx.is_blob_duplicate(&[8; 32]).await.unwrap());
    assert_eq!(false, idx.is_blob_duplicate(&[101; 32]).await.unwrap());

    idx.finalize_packfile(&mut packfile_handle, [0xf8; 12]).await.unwrap();

    assert_eq!(true, idx.is_blob_duplicate(&[8; 32]).await.unwrap());
    assert_eq!(false, idx.is_blob_duplicate(&[101; 32]).await.unwrap());

    idx.flush().await.unwrap();

    assert_eq!(true, idx.is_blob_duplicate(&[8; 32]).await.unwrap());
    assert_eq!(false, idx.is_blob_duplicate(&[101; 32]).await.unwrap());

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();

    assert_eq!(true, idx.is_blob_duplicate(&[8; 32]).await.unwrap());
    assert_eq!(false, idx.is_blob_duplicate(&[101; 32]).await.unwrap());
    assert_eq!([0xf8; 12], idx.find_packfile(&[7; 32]).await.unwrap().unwrap());
    assert_eq!(None, idx.find_packfile(&[102; 32]).await.unwrap());

    let mut packfile_handle = idx.begin_packfile();

    for i in 101..=200 {
        let blob_hash = [i; 32];
        idx.add_to_packfile(&mut packfile_handle, blob_hash).unwrap();
    }

    assert_eq!(true, idx.is_blob_duplicate(&[8; 32]).await.unwrap());
    assert_eq!(true, idx.is_blob_duplicate(&[105; 32]).await.unwrap());
    assert_eq!(false, idx.is_blob_duplicate(&[205; 32]).await.unwrap());

    idx.finalize_packfile(&mut packfile_handle, [0x8f; 12]).await.unwrap();
    idx.flush().await.unwrap();

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();
    assert_eq!(true, idx.is_blob_duplicate(&[8; 32]).await.unwrap());
    assert_eq!(true, idx.is_blob_duplicate(&[105; 32]).await.unwrap());
    assert_eq!(false, idx.is_blob_duplicate(&[205; 32]).await.unwrap());
    assert_eq!([0xf8; 12], idx.find_packfile(&[7; 32]).await.unwrap().unwrap());
    assert_eq!([0x8f; 12], idx.find_packfile(&[102; 32]).await.unwrap().unwrap());
    assert_eq!(None, idx.find_packfile(&[202; 32]).await.unwrap());
}

#[tokio::test]
async fn test_index_push() {
    let dir = format!("{}/idx2", std::env::temp_dir().to_str().unwrap());
    let keys = Keys::from_entropy([0; 32]);

    fs::remove_dir_all(dir.clone()).await.ok();

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();
    idx.push(&[0; 32], &[0; 12]).await.unwrap();
    idx.flush().await.unwrap();

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();
    assert_eq!([0; 12], idx.find_packfile(&[0; 32]).await.unwrap().unwrap());

    fs::remove_dir_all(dir.clone()).await.unwrap();

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();

    for i in 0..=250 {
        idx.push(&[i; 32], &[i; 12]).await.unwrap();
    }

    idx.flush().await.unwrap();

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();

    for i in 250..=0 {
        assert_eq!([i; 12], idx.find_packfile(&[i; 32]).await.unwrap().unwrap());
    }

    fs::remove_dir_all(dir.clone()).await.unwrap();
    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();

    for i in 0..=255 {
        idx.push(&[i; 32], &[i; 12]).await.unwrap();
        idx.flush().await.unwrap();
    }

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();
    for i in 255..=0 {
        assert_eq!([i; 12], idx.find_packfile(&[i; 32]).await.unwrap().unwrap());
    }

    fs::remove_dir_all(dir.clone()).await.unwrap();
}

#[tokio::test]
async fn test_index_push_rand() {
    let dir = format!("{}/idx3", std::env::temp_dir().to_str().unwrap());
    let entropy: Vec<u8> = (0..32).collect();
    let keys = Keys::from_entropy(entropy.try_into().unwrap());

    fs::remove_dir_all(dir.clone()).await.ok();

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();
    for i in 0..=50 {
        let mut rng = ChaCha8Rng::seed_from_u64(i);
        let mut blob_hash = [0; 32];
        let mut packfile_hash = [0; 12];

        rng.fill_bytes(&mut blob_hash);
        rng.fill_bytes(&mut packfile_hash);

        idx.push(&blob_hash, &packfile_hash).await.unwrap();
    }

    idx.flush().await.unwrap();

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();
    for i in 51..=500 {
        let mut rng = ChaCha8Rng::seed_from_u64(i);
        let mut blob_hash = [0; 32];
        let mut packfile_hash = [0; 12];

        rng.fill_bytes(&mut blob_hash);
        rng.fill_bytes(&mut packfile_hash);

        idx.push(&blob_hash, &packfile_hash).await.unwrap();
    }

    idx.flush().await.unwrap();

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();
    for i in 501..=60_000 {
        let mut rng = ChaCha8Rng::seed_from_u64(i);
        let mut blob_hash = [0; 32];
        let mut packfile_hash = [0; 12];

        rng.fill_bytes(&mut blob_hash);
        rng.fill_bytes(&mut packfile_hash);

        idx.push(&blob_hash, &packfile_hash).await.unwrap();
    }

    idx.flush().await.unwrap();

    let mut idx = BlobIndex::new(dir.clone(), keys.clone()).await.unwrap();

    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let mut values: Vec<u64> = (0..=60_000).collect();
    values.shuffle(&mut rng);

    for i in values {
        let mut rng = ChaCha8Rng::seed_from_u64(i);
        let mut blob_hash = [0; 32];
        let mut packfile_hash = [0; 12];

        rng.fill_bytes(&mut blob_hash);
        rng.fill_bytes(&mut packfile_hash);
        assert_eq!(packfile_hash, idx.find_packfile(&blob_hash).await.unwrap().unwrap());
    }

    fs::remove_dir_all(dir.clone()).await.unwrap();
}
