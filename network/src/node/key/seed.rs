use super::{ Mnemonic, Error };

use pluto_utils::bits::{ BitsIter, IterBits };

pub const SEED_NUM_BYTES: usize = 32;
pub const SEED_NUM_BITS: usize = SEED_NUM_BYTES * 8;
pub const CHECKSUM_NUM_BITS: usize = SEED_NUM_BITS / 32;

pub type Entropy = [u8; SEED_NUM_BYTES];

/// Seed used to initialise a PRNG for key generation.
///
/// Convert to and from a [`Mnemonic`] for easy storage.
#[derive(Eq, PartialEq, Clone)]
pub struct Seed {
    /// Random bytes acting as an entropy.
    entropy: Entropy,
    /// Checksum for verifying whether a passphrase is correct.
    checksum: u8
}

impl Seed {
    /// Converts a [`Mnemonic`] to a [`Seed`].
    ///
    /// Returns [`InvalidChecksum`](Error::InvalidChecksum) if
    /// the checksum doesn't match.
    pub fn from_mnemonic(mnemonic: Mnemonic) -> Result<Seed, Error> {
        let indexes = mnemonic.to_indexes();

        let bits: Vec<u8> = indexes.iter_n_bits(11).collect();

        let mut bytes: Vec<u8> = bits.chunks(8).map(|bits|
            bits.into_iter()
                .fold(
                    (0, 0),
                    |(count, value), bit| (count + 1, value | (bit << count))
                ).1
        ).collect();

        let checksum = bytes.pop().unwrap();

        let seed = Seed {
            entropy: bytes.try_into().unwrap(),
            checksum,
        };

        seed.verify_checksum()?;

        Ok(seed)
    }

    /// Converts a [`Seed`] to [`Mnemonic`].
    pub fn to_mnemonic(&self) -> Mnemonic {
        let bits: Vec<u8> = self.entropy.iter_bits()
            .chain(self.checksum.iter_bits())
            .collect();

        let indexes: Vec<u16> = bits.chunks(11).map(|bits| {
            let bits: [u8; 11] = bits.try_into().unwrap();
            let bits = bits.map(|b| b as u16);

            bits.into_iter()
                .fold(
                    (0, 0),
                    |(count, value), bit| (count + 1, value | (bit << count))
                ).1
        }).collect();

        // SAFETY: There is no way to obtain invalid indexes
        // from any entropy. Hence, we can unwrap.
        Mnemonic::from_indexes(indexes.try_into().unwrap()).unwrap()
    }

    /// Verifies whether the checksum is valid.
    ///
    /// Returns [`InvalidChecksum`](Error::InvalidChecksum)
    /// error if it doesn't match.
    pub fn verify_checksum(&self) -> Result<(), Error> {
        let hash = ring::digest::digest(&ring::digest::SHA256, &self.entropy);

        if hash.as_ref()[0] != self.checksum {
            Err(Error::InvalidChecksum)
        } else { Ok(()) }
    }

    /// Converts random bytes into a [`Seed`], and calculates its checksum.
    pub fn from_entropy(entropy: Entropy) -> Seed {
        let hash = ring::digest::digest(&ring::digest::SHA256, &entropy);
        assert_eq!(CHECKSUM_NUM_BITS, 8, "checksum sizes other than 8 bits are currently not supported");

        Seed {
            entropy,
            checksum: hash.as_ref()[0]
        }
    }

    /// Returns seed entropy.
    pub fn entropy(&self) -> &Entropy {
        &self.entropy
    }
}

impl std::fmt::Debug for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Seed")
            // Output entropy as a string of 2-digit hex numbers.
            .field("entropy", &self.entropy.iter().copied().fold(String::new(), |mut s, b| {
                s.push_str(&format!("{b:02x}"));
                s
            }))
            // Output checksum as a 2-digit hex number.
            .field("checksum", &format!("{:02x}", self.checksum))
            .finish()
    }
}

impl TryFrom<Mnemonic> for Seed {
    type Error = Error;

    fn try_from(m: Mnemonic) -> Result<Self, Self::Error> {
        Self::from_mnemonic(m)
    }
}

impl Into<Mnemonic> for Seed {
    fn into(self) -> Mnemonic {
        self.to_mnemonic()
    }
}