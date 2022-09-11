pub mod mnemonic; pub use mnemonic::*;
mod seed; pub use seed::*;

use rand_chacha::{ ChaCha20Rng,
    rand_core::{ RngCore, SeedableRng }
};

use x25519_dalek::{ PublicKey, StaticSecret };

use hkdf::Hkdf;
use sha2::Sha256;

type SymmetricKey = [u8; 32];

/// Represents cryptographic keys used for asymmetric authentication/encryption and symmetric data encryption.
/// Includes a [X25519](x25519_dalek) [public](PublicKey) and [private](StaticSecret) key pair,
/// and a master symmetric key, used for derivation of symmetric keys for individual nodes using a KDF.
///
/// These keys are generated from a seeded PRNG ([`ChaCha20`](ChaCha20Rng)), meaning
/// they can be recreated given the initial entropy.
#[derive(Clone)]
pub struct Keys {
    seed: Seed,
    public_key: PublicKey,
    private_key: StaticSecret,
    symmetric_master_key: SymmetricKey
}

impl Keys {
    /// Generates a new entropy and keys.
    ///
    /// This function uses the system's RNG to generate the initial seed.
    /// See [`getrandom`](getrandom::getrandom) for details.
    pub fn generate() -> Self {
        let mut entropy: Entropy = Default::default();
        getrandom::getrandom(&mut entropy).expect("Getrandom failed");

        Self::from_entropy(entropy)
    }

    /// Creates the [`X25519`](x25519_dalek) key pair and a symmetric master key given the initial entropy.
    pub fn from_entropy(entropy: Entropy) -> Self {
        // Seed ChaCha20 CSPRNG with our entropy to deterministically generate keys
        let mut rng = ChaCha20Rng::from_seed(entropy);

        let mut keypair_seed: Entropy = Default::default();

        // First get randomness for generating a X25519 keypair
        rng.fill_bytes(&mut keypair_seed);

        let private_key = StaticSecret::from(keypair_seed);
        let public_key = PublicKey::from(&private_key);

        let mut symmetric_master_key: SymmetricKey = [0; 32];

        // Now get randomness for symmetric encryption master key
        rng.fill_bytes(&mut symmetric_master_key);

        Self {
            seed: Seed::from_entropy(rng.get_seed()),
            public_key,
            private_key,
            symmetric_master_key
        }
    }

    /// Creates the [`X25519`](x25519_dalek) key pair and a symmetric master key from a [`Seed`] object.
    pub fn from_seed(seed: Seed) -> Self {
        let entropy = seed.entropy();

        Self::from_entropy(*entropy)
    }

    /// Derives a new key for symmetric encryption from master key, given a info parameter
    /// (e.g. pubkey of a node we want to back up to).
    pub fn derive_symmetric_key(&self, info: &[u8]) -> SymmetricKey {
        let mut derived_key: SymmetricKey = [0; 32];

        // Initialize HKDF with our master key -- PRK
        let kdf = Hkdf::<Sha256>::from_prk(&self.symmetric_master_key).unwrap();

        kdf.expand(info, &mut derived_key).unwrap();
        derived_key
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
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
fn test_mnemonic_checksum() {
    let keys = Keys::generate();
    let seed = keys.seed().clone();
    let mnemonic_a = seed.to_mnemonic();
    let mut indexes = mnemonic_a.to_indexes();
    indexes[7] = (indexes[7] + 1) % (WORDLIST_LENGTH as u16);
    let mnemonic_b = Mnemonic::from_indexes(indexes).unwrap();
    assert_eq!(Seed::from_mnemonic(mnemonic_b), Err(Error::InvalidChecksum));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_symmetric_key_derivation() {
    let keys_a = Keys::generate();
    let seed = keys_a.seed().clone();
    let info_a = [0x00; 32];
    let info_b = [0xff; 32];
    let derived_key_a = keys_a.derive_symmetric_key(&info_a);
    let derived_key_b = keys_a.derive_symmetric_key(&info_b);
    let keys_b = Keys::from_seed(seed);
    let derived_key_c = keys_b.derive_symmetric_key(&info_a);
    let derived_key_d = keys_b.derive_symmetric_key(&info_b);
    assert_eq!(derived_key_a, derived_key_c);
    assert_eq!(derived_key_b, derived_key_d);
    assert_ne!(derived_key_a, derived_key_b);
}
