use aes_gcm::{AeadInOut, Aes256Gcm, Key, KeyInit, Nonce, Tag, aead::consts::U12, aes::cipher::InOutBuf};
use embassy_rp::clocks::RoscRng;
use pbkdf2::{pbkdf2_hmac, sha2::{Sha256}};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::storage::{constants::{AES_KEY_LEN, AES_NONCE_SIZE, PBKDF2_ITERATIONS, PBKDF2_SALT_LEN}, errors::{CryptoError, CryptoResult}};



fn rand_array<const N: usize>() -> [u8; N] {
    core::array::from_fn(|_| RoscRng::next_u8())
}


pub type SaltBuffer = [u8; PBKDF2_SALT_LEN];


pub struct Passphrase([u8; AES_KEY_LEN]);

impl Passphrase {
    pub fn as_key(&self) -> &Key<Aes256Gcm> {
        match self.0.as_slice().try_into() {
            Ok(key) => key,
            Err(_) => unreachable!("Only fails if key size != AES_KEY_LEN, which will never fail")
        }
    }
}



// Wrapper around a buffer that contains a passphrase
// Struct will zeroize on drop so data does not linger in RAM
pub struct UnbuiltPassphrase<'a>(&'a mut [u8]);

impl<'a> UnbuiltPassphrase<'a> {
    pub fn as_slice(&self) -> &[u8] {
        self.0
    }

    pub fn build(self) -> (Passphrase, SaltBuffer) {
        let salt = rand_array();
        (self.build_with_salt(salt), salt)
    }

    pub fn build_with_salt(self, salt: SaltBuffer) -> Passphrase {
        let mut key_bytes = [0u8; AES_KEY_LEN];
        pbkdf2_hmac::<Sha256>(
            self.as_slice(),
            &salt,
            PBKDF2_ITERATIONS,
            &mut key_bytes
        );
        Passphrase(key_bytes)
    }
}

impl<'a> Drop for UnbuiltPassphrase<'a> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}


pub trait PlaintextBuffer {
    fn len(&self) -> usize;
    fn as_slice(&self) -> &[u8];
}

// Wrapper around a buffer that contains a plaintext buffer
// Struct will zeroize on drop so data does not linger in RAM
pub struct Plaintext<'a>(&'a mut [u8]);

impl<'a> PlaintextBuffer for Plaintext<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn as_slice(&self) -> &[u8] {
        self.0
    }
}

impl<'a> Drop for Plaintext<'a> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub struct StaticPlaintext(&'static [u8]);

impl StaticPlaintext {
    pub fn new(plaintext: &'static [u8]) -> Self {
        Self(plaintext)
    }
}

impl PlaintextBuffer for StaticPlaintext {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn as_slice(&self) -> &[u8] {
        self.0
    }
}

#[derive(ZeroizeOnDrop)]
pub struct OwnedPlaintext<const N: usize> {
    plaintext: [u8; N],
    len: usize
}

impl<const N: usize> OwnedPlaintext<N> {
    pub fn new(plaintext: [u8; N], len: usize) -> Self {
        Self {
            plaintext,
            len
        }
    }
}

impl<const N: usize> PlaintextBuffer for OwnedPlaintext<N> {
    fn len(&self) -> usize {
        self.len
    }

    fn as_slice(&self) -> &[u8] {
        &self.plaintext[..self.len]
    }
}


/*
 * nonce: 12 bytes
 * tag: 16 bytes
 * data_len: 1 byte
 * data: MAX_TEXT_SIZE bytes
 * -------------------------
 * total: 29 + MAX_TEXT_SIZE
 */
#[repr(C)]
pub struct Ciphertext<const MAX_TEXT_SIZE: usize> {
    nonce: Nonce<U12>,
    tag: Tag,
    data_len: u8,
    data: [u8; MAX_TEXT_SIZE],
}

impl<const MAX_TEXT_SIZE: usize> Ciphertext<MAX_TEXT_SIZE> {
    pub fn new<P: PlaintextBuffer>(plaintext: P, passphrase: Passphrase) -> CryptoResult<Self> {
        let key = passphrase.as_key();
        let cipher = Aes256Gcm::new(&key);
        let nonce_bytes: [u8; AES_NONCE_SIZE] = rand_array();
        let nonce: Nonce<U12> = match nonce_bytes.as_slice().try_into() {
            Ok(nonce) => nonce,
            Err(_) => unreachable!("Only fails if nonce_bytes size != AES_NONCE_SIZE, which will never fail")
        };
        let mut data = [0u8; MAX_TEXT_SIZE];
        let data_len = usize::min(MAX_TEXT_SIZE, plaintext.len());
        let data_len = usize::min(data_len, u8::MAX as usize);

        let tag = cipher
            .encrypt_inout_detached(
                &nonce,
                b"",
                InOutBuf::from(&mut data[..data_len])
            )
            .map_err(|_| CryptoError::EncryptionError)?;

        Ok(Self{
            nonce,
            tag,
            data_len: data_len as u8,
            data
        })
    }

    pub fn data_len(&self) -> u8 {
        self.data_len
    }

    pub fn decrypt(
        &self,
        passphrase: &Passphrase,
    ) -> CryptoResult<OwnedPlaintext<MAX_TEXT_SIZE>> {
        let key = passphrase.as_key();
        let cipher = Aes256Gcm::new(&key);

        let mut plaintext = [0u8; MAX_TEXT_SIZE];
        let valid_len = usize::min(self.data_len as usize, MAX_TEXT_SIZE);

        cipher
            .decrypt_inout_detached(
                &self.nonce,
                b"",
                InOutBuf::from(&mut plaintext[..valid_len]),
                &self.tag
            )
            .map(|_| OwnedPlaintext::new(plaintext, valid_len))
            .map_err(|_| CryptoError::DecryptionError)
    }
}
