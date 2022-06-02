// const WORDLIST_EN: &[&'static str] = pluto_macros::wordlist!("network/src/node/key/wordlists/english.txt");
lazy_static::lazy_static!{
    static ref WORDLIST_EN: Vec<&'static str> = {
        let s = include_str!("wordlists/english.txt");

        let v: Vec<&'static str> = s.split_whitespace().collect();

        assert_eq!(v.len(), WORDLIST_LENGTH as usize, "Invalid wordlist file.");

        v
    };
}

/// Number of words in the passphrase string.
pub const PASSPHRASE_LENGTH: usize = 24;
/// Total number of words which can be used in a mnemonic.
pub const WORDLIST_LENGTH: u16 = 2048;

/// A 24 word mnemonic which represents the initial entropy used
/// to [seed](super::Seed) a PRNG for generating keys.
#[derive(Debug, PartialEq, Eq)]
pub struct Mnemonic {
    words: [String; PASSPHRASE_LENGTH]
}

impl Mnemonic {
    /// Create a new [`Mnemonic`] from a whitespace-separated passphrase string.
    ///
    /// # Errors
    /// - [`InvalidLength`](Error::InvalidLength) if the inputted string does not
    /// contain the correct amount of words.
    ///
    /// - [`InvalidWord`](Error::InvalidWord) if the inputted string contains invalid
    /// words, which do not exist in the wordlist.
    pub fn from_passphrase(passphrase: String) -> Result<Mnemonic, Error> {
        let vector: Vec<String> = passphrase.split_whitespace()
            .map(ToOwned::to_owned)
            .collect();

        let array = vector.try_into().map_err(|_| Error::InvalidLength)?;

        let mnemonic = Mnemonic {
            words: array
        };

        mnemonic.verify_words()?;

        Ok(mnemonic)
    }

    /// Returns a [`Mnemonic`] as a space-separated passphrase string.
    pub fn to_passphrase(&self) -> String {
        self.words.join(" ")
    }

    /// Converts indexes of words in the wordlist to a [`Mnemonic`].
    ///
    /// Returns [`InvalidIndex`](Error::InvalidIndex) on the first index
    /// which is out of range.
    pub fn from_indexes(indexes: [u16; PASSPHRASE_LENGTH]) -> Result<Mnemonic, Error> {
        let mut words: Vec<String> = Vec::with_capacity(PASSPHRASE_LENGTH);

        for index in indexes.into_iter() {
            if index >= WORDLIST_LENGTH { return Err(Error::InvalidIndex(index))}
            words.push(WORDLIST_EN[index as usize].to_owned());
        }

        Ok(Mnemonic {
             words: words.try_into().unwrap()
        })
    }

    /// Converts words in [`Mnemonic`] into their respective indexes in the wordlist.
    pub fn to_indexes(&self) -> [u16; PASSPHRASE_LENGTH] {
        self.words.iter()
            .map(|word| WORDLIST_EN.binary_search(&&**word).unwrap() as u16)
            .collect::<Vec<u16>>()
            .try_into().unwrap()
    }

    /// Verifies that all words in the [`Mnemonic`] are valid
    /// words in the wordlist.
    ///
    /// Returns [`InvalidWord`](Error::InvalidWord) on the
    /// first appearing invalid word.
    pub fn verify_words(&self) -> Result<(), Error> {
        for word in &self.words {
            match WORDLIST_EN.binary_search(&&**word) {
                Ok(_) => {},
                Err(_) => return Err(Error::InvalidWord(word.clone()))
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_passphrase())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Inputted word is invalid.
    InvalidWord(String),
    /// Inputted index is out of range.
    InvalidIndex(u16),
    /// Invalid word count.
    InvalidLength,
    /// Checksum does not match.
    InvalidChecksum,
}