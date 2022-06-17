pub mod mnemonic; pub use mnemonic::*;
mod seed; pub use seed::*;

use rand_chacha::{ ChaCha20Rng,
    rand_core::{ RngCore, SeedableRng }
};

use x25519_dalek::{ PublicKey, StaticSecret };

/// Represents an [X25519](x25519_dalek) [public](PublicKey) and [private](StaticSecret) key pair for authentication.
///
/// These keys are generated from a seeded PRNG ([`ChaCha20`](ChaCha20Rng)), meaning
/// they can be recreated given the initial entropy.
pub struct Keys {
    seed: Seed,
    public_key: PublicKey,
    private_key: StaticSecret,
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

    /// Creates the [`X25519`](x25519_dalek) key pair given the initial entropy.
    pub fn from_entropy(entropy: Entropy) -> Self {
        let mut rng = ChaCha20Rng::from_seed(entropy);

        let mut keypair_seed: Entropy = Default::default();
        rng.fill_bytes(&mut keypair_seed);

        let private_key = StaticSecret::from(keypair_seed);
        let public_key = PublicKey::from(&private_key);

        Self {
            seed: Seed::from_entropy(rng.get_seed()),
            public_key,
            private_key,
        }
    }

    pub fn from_seed(seed: Seed) -> Self {
        let entropy = seed.entropy();
        
        Self::from_entropy(*entropy)
    }

    /// Returns the [X25519](x25519_dalek) [public key](PublicKey).
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the [X25519](x25519_dalek) [private key](StaticSecret).
    pub fn private_key(&self) -> &StaticSecret {
        &self.private_key
    }

    /// Returns the [`Seed`] used to generate the keys.
    pub fn seed(&self) -> &Seed {
        &self.seed
    }
}

impl std::fmt::Debug for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keys")
            .field("seed", &self.seed)
            .field("public_key", &self.public_key)
            .finish()
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
