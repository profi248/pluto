mod mnemonic; pub use mnemonic::*;
mod seed; pub use seed::*;

use rand_chacha::{ ChaCha20Rng,
    rand_core::{ RngCore, SeedableRng }
};

use ring::signature::Ed25519KeyPair;

pub struct Keys {
    pub entropy: [u8; 32],
    pub pair: Ed25519KeyPair,
}

impl std::fmt::Debug for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keys")
            .field("entropy", &format!("{:x?}", self.entropy))
            .field("pair", &self.pair)
            .finish()
    }
}

impl Keys {
    pub fn generate() -> Self {
        let mut entropy: [u8; 32] = Default::default();
        getrandom::getrandom(&mut entropy).expect("Getrandom failed");

        Self::from_entropy(entropy)
    }

    pub fn from_entropy(entropy: [u8; 32]) -> Self {
        let mut rng = ChaCha20Rng::from_seed(entropy);

        let mut keypair_seed: [u8; 32] = Default::default();
        rng.fill_bytes(&mut keypair_seed);

        let pair = Ed25519KeyPair::from_seed_unchecked(&keypair_seed)
            .expect("Failed to generate keypair");

        Self {
            entropy: rng.get_seed(),
            pair,
        }
    }
}

#[test]
fn test_mnemonic() {
    for _ in 0..10000 {
        let keys = Keys::generate();
        let seed_a = Seed::from_entropy(keys.entropy);
        let mnemonic = seed_a.to_mnemonic().unwrap();
        let seed_b = Seed::from_mnemonic(mnemonic).unwrap();
        assert_eq!(seed_a, seed_b);
    }
}

#[test]
fn test_mnemonic_checksum() {
    let keys = Keys::generate();
    let seed = Seed::from_entropy(keys.entropy);
    let mnemonic_a = seed.to_mnemonic().unwrap();
    let mut indexes = mnemonic_a.to_indexes();
    indexes[7] = 25;
    let mnemonic_b = Mnemonic::from_indexes(indexes).unwrap();
    assert_eq!(Seed::from_mnemonic(mnemonic_b), Err(Error::InvalidChecksum));
}
