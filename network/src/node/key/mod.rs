mod mnemonic; pub use mnemonic::*;
mod seed; pub use seed::*;

use rand_chacha::{ ChaCha20Rng,
    rand_core::{ RngCore, SeedableRng }
};

use ring::signature::Ed25519KeyPair;

/// Represents an [`Ed25519`](Ed25519KeyPair) public and private key pair for authentication.
///
/// These keys are generated from a seeded PRNG ([`ChaCha20`](ChaCha20Rng)), meaning
/// they can be recreated given the initial entropy.
#[derive(Debug)]
pub struct Keys {
    seed: Seed,
    pair: Ed25519KeyPair,
}

impl Keys {
    /// Generates a new entropy and key pair.
    ///
    /// This function uses the system's RNG to generate the initial seed.
    /// See [`getrandom`](getrandom::getrandom) for details.
    pub fn generate() -> Self {
        let mut entropy: Entropy = Default::default();
        getrandom::getrandom(&mut entropy).expect("Getrandom failed");

        Self::from_entropy(entropy)
    }

    /// Recreates the [`Ed25519`](Ed25519KeyPair) key pair given the initial entropy.
    pub fn from_entropy(entropy: Entropy) -> Self {
        let mut rng = ChaCha20Rng::from_seed(entropy);

        let mut keypair_seed: Entropy = Default::default();
        rng.fill_bytes(&mut keypair_seed);

        let pair = Ed25519KeyPair::from_seed_unchecked(&keypair_seed)
            .expect("Failed to generate keypair");

        Self {
            seed: Seed::from_entropy(rng.get_seed()),
            pair,
        }
    }

    /// Returns the [`Ed25519`](Ed25519KeyPair).
    pub fn pair(&self) -> &Ed25519KeyPair {
        &self.pair
    }

    /// Returns the [`Seed`] used to generate the keys.
    pub fn seed(&self) -> &Seed {
        &self.seed
    }
}

#[test]
fn test_mnemonic() {
    for _ in 0..10000 {
        let keys = Keys::generate();
        let seed_a = keys.seed().clone();
        let mnemonic = seed_a.to_mnemonic();
        let seed_b = Seed::from_mnemonic(mnemonic).unwrap();
        assert_eq!(seed_a, seed_b);
    }
}

#[test]
fn test_mnemonic_checksum() {
    let keys = Keys::generate();
    let seed = keys.seed().clone();
    let mnemonic_a = seed.to_mnemonic();
    let mut indexes = mnemonic_a.to_indexes();
    indexes[7] = (indexes[7] + 1) % (WORDLIST_LENGTH as u16);
    let mnemonic_b = Mnemonic::from_indexes(indexes).unwrap();
    assert_eq!(Seed::from_mnemonic(mnemonic_b), Err(Error::InvalidChecksum));
}
