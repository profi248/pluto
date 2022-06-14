// HKPE Protocol - https://www.rfc-editor.org/rfc/rfc9180.html

// Key Encapsulation Mechanism
use x25519_dalek::{ PublicKey, StaticSecret };

// Key Derivation Function
use hkdf::Hkdf;
use sha2::Sha256;

// Authenticated Encryption with Associated Data
use aes_gcm::{ Aes256Gcm, NewAead, aead::Aead };

use std::marker::PhantomData;

use crate::{ key::Keys, handler::Message, prelude::* };

pub type Nonce = [u8; 12];
pub type Salt = [u8; 32];
pub type Key = [u8; 32];

/// A wrapper around encrypted or unencrypted messages.
pub enum MessageVariant<M: MessageTrait> {
    Encrypted(EncryptedMessage<M>),
    Unencrypted(M)
}

impl<M: MessageTrait> MessageVariant<M> {
    /// Serialises the inner message to bytes.
    pub fn write_to_bytes(&self) -> protobuf::Result<Vec<u8>> {
        match self {
            MessageVariant::Encrypted(m) => m.write_to_bytes(),
            MessageVariant::Unencrypted(m) => m.write_to_bytes(),
        }
    }

    /// Extracts the encrypted message.
    ///
    /// Returns `Self` as an error, as this method is consuming.
    pub fn encrypted(self) -> StdResult<EncryptedMessage<M>, Self> {
        match self {
            MessageVariant::Encrypted(m) => Ok(m),
            MessageVariant::Unencrypted(_) => Err(self),
        }
    }

    /// Extracts the unencrypted message.
    ///
    /// Returns `Self` as an error, as this method is consuming.
    pub fn unencrypted(self) -> StdResult<M, Self> {
        match self {
            MessageVariant::Encrypted(_) => Err(self),
            MessageVariant::Unencrypted(m) => Ok(m),
        }
    }
}

impl<M: MessageTrait> From<EncryptedMessage<M>> for MessageVariant<M> {
    fn from(m: EncryptedMessage<M>) -> Self {
        Self::Encrypted(m)
    }
}

impl<M: MessageTrait> From<M> for MessageVariant<M> {
    fn from(m: M) -> Self {
        Self::Unencrypted(m)
    }
}

/// A wrapper around an [`EncryptedMessage`](crate::protos::shared::EncryptedMessage) proto struct.
///
/// Provides helper methods to decrypt messages and return inner message type.
pub struct EncryptedMessage<M: MessageTrait> {
    inner: crate::protos::shared::EncryptedMessage,
    _phantom: PhantomData<M>,
}

impl<M: MessageTrait> From<crate::protos::shared::EncryptedMessage> for EncryptedMessage<M> {
    fn from(m: crate::protos::shared::EncryptedMessage) -> Self {
        Self {
            inner: m,
            _phantom: PhantomData
        }
    }
}

impl<M: MessageTrait> EncryptedMessage<M> {
    /// Decrypts the message into the inner message type.
    pub fn decrypt(self, private_key: &StaticSecret) -> Option<M> {
        let ephemeral_public_key: Key = self.inner.ephemeral_pubkey.as_slice().try_into().ok()?;

        let symmetric_key = decapsulate(
            ephemeral_public_key.into(),
            private_key,
            self.inner.salt.as_slice().try_into().ok()?,
        );


        let bytes = decrypt(&self.inner.inner_message, self.inner.nonce.try_into().ok()?, symmetric_key)?;

        Message::new(bytes.into()).unencrypted().ok()
    }

    /// Decrypts the message into the inner message type,
    /// verifying that the owner of the public key encrypted this message.
    pub fn decrypt_authenticated(self, recipient_private_key: &StaticSecret, sender_public_key: &PublicKey) -> Option<M> {
        let ephemeral_public_key: Key = self.inner.ephemeral_pubkey.as_slice().try_into().ok()?;

        let symmetric_key = decapsulate_authenticated(
            sender_public_key,
            ephemeral_public_key.into(),
            recipient_private_key,
            self.inner.salt.as_slice().try_into().ok()?,
        );

        let bytes = decrypt(&self.inner.inner_message, self.inner.nonce.try_into().ok()?, symmetric_key)?;

        Message::new(bytes.into()).unencrypted().ok()
    }

    /// Serialises the inner message to bytes.
    pub fn write_to_bytes(&self) -> protobuf::Result<Vec<u8>> {
        self.inner.write_to_bytes()
    }
}

pub trait Encrypt: MessageTrait {
    /// Encrypts this message using the recipient's public key.
    fn encrypt(self, recipient_public_key: &PublicKey) -> EncryptedMessage<Self>;

    /// Encrypts this message using the recipient's public key,
    /// and signs the message with your private key.
    fn encrypt_authenticated(self,
        recipient_public_key: &PublicKey,
        sender_private_key: &StaticSecret,
        sender_public_key: &PublicKey
    ) -> EncryptedMessage<Self>;
}

impl<M: MessageTrait> Encrypt for M {
    fn encrypt(self, recipient_public_key: &PublicKey) -> EncryptedMessage<Self> {
        let Encapsulation {
            ephemeral_public_key,
            symmetric_key,
            salt
        } = encapsulate(recipient_public_key);

        let bytes = self.write_to_bytes().unwrap();

        let Encryption { ciphertext, nonce } = encrypt(&bytes, symmetric_key);

        let mut inner = crate::protos::shared::EncryptedMessage::default();
        inner.ephemeral_pubkey = ephemeral_public_key.as_bytes().to_vec();
        inner.salt = salt.to_vec();
        inner.nonce = nonce.to_vec();
        inner.inner_message = ciphertext;

        EncryptedMessage {
            inner,
            _phantom: PhantomData
        }
    }

    fn encrypt_authenticated(self, recipient_public_key: &PublicKey, sender_private_key: &StaticSecret, sender_public_key: &PublicKey) -> EncryptedMessage<Self> {
        let Encapsulation {
            ephemeral_public_key,
            symmetric_key,
            salt
        } = encapsulate_authenticated(recipient_public_key, sender_private_key);

        let bytes = self.write_to_bytes().unwrap();

        let Encryption { ciphertext, nonce } = encrypt(&bytes, symmetric_key);

        let mut inner = crate::protos::shared::EncryptedMessage::default();
        inner.ephemeral_pubkey = ephemeral_public_key.as_bytes().to_vec();
        inner.sender_pubkey = sender_public_key.as_bytes().to_vec();
        inner.salt = salt.to_vec();
        inner.nonce = nonce.to_vec();
        inner.inner_message = ciphertext;

        EncryptedMessage {
            inner,
            _phantom: PhantomData
        }
    }
}

mod encryption {
    use super::*;

    pub struct Encapsulation {
        pub ephemeral_public_key: PublicKey,
        pub symmetric_key: [u8; 32],
        pub salt: [u8; 32],
    }

    pub fn encapsulate(recipient_public_key: &PublicKey) -> Encapsulation {
        // Generate a new ephemeral X25519 key pair.
        // The crate uses an old version of rand so we can't
        // use the nice EphemeralSecret API. :(
        let (ephemeral_public_key, ephemeral_private_key) = {
            let keys = Keys::generate();

            (keys.public_key().clone(), keys.private_key().clone())
        };

        // Perform a Diffie-Hellman. This becomes our input key material.
        let shared_secret = ephemeral_private_key.diffie_hellman(recipient_public_key).to_bytes();

        // Generate a random salt.
        let mut salt = [0; 32];
        getrandom::getrandom(&mut salt).unwrap();

        // Perform Expand operation using empty info.
        let symmetric_key = key_derivation_function(salt, &shared_secret);

        Encapsulation {
            ephemeral_public_key,
            symmetric_key,
            salt
        }
    }

    pub fn encapsulate_authenticated(recipient_public_key: &PublicKey, sender_private_key: &StaticSecret) -> Encapsulation {
        // Generate a new ephemeral X25519 key pair.
        // The crate uses an old version of rand so we can't
        // use the nice EphemeralSecret API. :(
        let (ephemeral_public_key, ephemeral_private_key) = {
            let keys = Keys::generate();

            (keys.public_key().clone(), keys.private_key().clone())
        };

        let shared_secret = {
            // Perform the first Diffie-Hellman.
            let a = ephemeral_private_key.diffie_hellman(recipient_public_key).to_bytes();
            // Perform the second Diffie-Hellman.
            let b = sender_private_key.diffie_hellman(recipient_public_key).to_bytes();

            // Concat Diffie-Hellman results.
            let mut s = [0; 64];

            s[..32].copy_from_slice(&a);
            s[32..].copy_from_slice(&b);

            s
        };

        // Generate a random salt.
        let mut salt = [0; 32];
        getrandom::getrandom(&mut salt).unwrap();

        // Perform Expand operation using empty info.
        let symmetric_key = key_derivation_function(salt, &shared_secret);

        Encapsulation {
            ephemeral_public_key,
            symmetric_key,
            salt
        }
    }

    pub fn decapsulate(ephemeral_public_key: PublicKey, recipient_private_key: &StaticSecret, salt: Salt) -> Key {
        let shared_secret = recipient_private_key.diffie_hellman(&ephemeral_public_key).to_bytes();

        key_derivation_function(salt, &shared_secret)
    }

    pub fn decapsulate_authenticated(
        sender_public_key: &PublicKey,
        ephemeral_public_key: PublicKey,
        recipient_private_key: &StaticSecret,
        salt: Salt
    ) -> Key {
        let shared_secret = {
            // Perform the first Diffie-Hellman.
            let a = recipient_private_key.diffie_hellman(&ephemeral_public_key).to_bytes();
            // Perform the second Diffie-Hellman.
            let b = recipient_private_key.diffie_hellman(sender_public_key).to_bytes();

            // Concat Diffie-Hellman results.
            let mut s = [0; 64];

            s[..32].copy_from_slice(&a);
            s[32..].copy_from_slice(&b);

            s
        };

        key_derivation_function(salt, &shared_secret)
    }

    pub fn key_derivation_function(salt: Salt, shared_secret: &[u8]) -> Key {
        // Perform Extract operation using salt and
        // Diffie-Hellman shared secret as input key material.
        let hkdf = Hkdf::<Sha256>::new(Some(&salt), shared_secret);

        // Perform Expand operation using empty info.
        let mut expanded_key = [0; 32];
        hkdf.expand(&[], &mut expanded_key).unwrap();

        expanded_key
    }

    pub struct Encryption {
        pub ciphertext: Vec<u8>,
        pub nonce: Nonce,
    }

    pub fn encrypt(data: &[u8], symmetric_key: Key) -> Encryption {
        let aes = Aes256Gcm::new(&symmetric_key.into());

        let mut nonce = [0; 12];
        getrandom::getrandom(&mut nonce).unwrap();

        let ciphertext = aes.encrypt(&nonce.into(), data).unwrap();

        Encryption {
            ciphertext,
            nonce
        }
    }

    pub fn decrypt(ciphertext: &[u8], nonce: Nonce, symmetric_key: Key) -> Option<Vec<u8>> {
        let aes = Aes256Gcm::new(&symmetric_key.into());

        aes.decrypt(&nonce.into(), ciphertext).ok()
    }
}
use encryption::*;
