use sha2::{ Sha256, Digest };
use rayon::prelude::*;

pub struct PowChallenge;

impl PowChallenge {
    pub fn verify(challenge: &[u8; 16], nonce: &[u8; 16], difficulty: u32) -> bool {
        let mut concat = [0u8; 32];
        concat[..16].copy_from_slice(challenge);
        concat[16..].copy_from_slice(nonce);

        let hash: [u8; 32] = Sha256::digest(&concat).into();

        let check = u32::from_be_bytes(hash[..4].try_into().unwrap());

        check < difficulty
    }

    pub fn compute(challenge: &[u8; 16], difficulty: u32) -> [u8; 16] {
        let nonce = (0..u128::MAX).into_par_iter().find_any(|nonce| {
            let mut concat = [0u8; 32];
            concat[..16].copy_from_slice(challenge);
            concat[16..].copy_from_slice(&nonce.to_be_bytes());
            let hash_computed: [u8; 32] = Sha256::digest(&concat).into();
            let check = u32::from_be_bytes(hash_computed[..4].try_into().unwrap());

            check < difficulty
        });

        nonce.unwrap().to_be_bytes()
    }
}
