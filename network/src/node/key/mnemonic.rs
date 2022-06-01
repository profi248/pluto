use crate::node::key::Error::InvalidIndex;

// const WORDLIST_EN: &[&'static str] = pluto_macros::wordlist!("network/src/node/key/wordlists/english.txt");
lazy_static::lazy_static!{
    static ref WORDLIST_EN: Vec<&'static str> = {
        let s = include_str!("wordlists/english.txt");

        s.split_whitespace().collect()
    };
}

const PASSPHRASE_LENGTH: usize = 24;

#[derive(Debug, PartialEq, Eq)]
pub struct Mnemonic {
    words: [String; PASSPHRASE_LENGTH]
}

impl Mnemonic {
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

    pub fn to_passphrase(&self) -> String {
        self.words.join(" ")
    }

    pub fn from_indexes(indexes: [u16; PASSPHRASE_LENGTH]) -> Result<Mnemonic, Error> {
        let mut words: Vec<String> = Vec::with_capacity(PASSPHRASE_LENGTH);

        for index in indexes.into_iter() {
            if index > 2047 { return Err(InvalidIndex(index))}
            words.push(WORDLIST_EN[index as usize].to_owned());
        }

        Ok(Mnemonic {
             words: words.try_into().unwrap()
        })
    }

    pub fn to_indexes(&self) -> [u16; PASSPHRASE_LENGTH] {
        self.words.iter()
            .map(|word| WORDLIST_EN.binary_search(&&**word).unwrap() as u16)
            .collect::<Vec<u16>>()
            .try_into().unwrap()
    }

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
    InvalidWord(String),
    InvalidIndex(u16),
    InvalidLength,
    InvalidChecksum,
}