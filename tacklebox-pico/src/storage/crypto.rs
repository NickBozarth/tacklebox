use aes_gcm::{Aes256Gcm, Key};
use pbkdf2::{pbkdf2_hmac, sha2::Sha256};

use crate::storage::consts::{AES256GCM_KEY_LEN, AES256GCM_NONCE_LEN, AES256GCM_TAG_LEN, PBKDF2_ITERATIONS, PBKDF2_SALT_LEN};

pub struct Ciphertext<const N: usize> {
    nonce: [u8; AES256GCM_NONCE_LEN],
    tag: [u8; AES256GCM_TAG_LEN],
    ciphertext: [u8; N]
}

impl<const N: usize> Ciphertext<N> {
    pub fn from_parts(
        nonce: [u8; AES256GCM_NONCE_LEN],
        tag: [u8; AES256GCM_TAG_LEN],
        ciphertext: [u8; N]
    ) -> Self {
        Self { nonce, tag, ciphertext }
    }

    pub fn as_parts(&self) -> ([u8; AES256GCM_NONCE_LEN], [u8; AES256GCM_TAG_LEN], [u8; N]) {
        (self.nonce, self.tag, self.ciphertext)
    }

    pub fn decrypt(&self, key: &Key<Aes256Gcm>) -> ! {
        todo!()
    }
}

pub struct Aes256GcmKeyBytes([u8; AES256GCM_KEY_LEN]);

impl Aes256GcmKeyBytes {
    pub fn new(passphrase: &[u8], salt: &[u8; PBKDF2_SALT_LEN]) -> Self {
        let mut key_buf = [0u8; AES256GCM_KEY_LEN];
        pbkdf2_hmac::<Sha256>(
            passphrase,
            salt,
            PBKDF2_ITERATIONS,
            &mut key_buf
        );
        Self(key_buf)
    }

    pub fn new_unsalted(passphrase: &[u8]) -> (Self, [u8; PBKDF2_SALT_LEN]) {

    }

    pub fn as_key(&self) -> Key<Aes256Gcm> {
        self.0
            .as_slice()
            .try_into()
            .expect("Only fails if self.0.len() != AES256GCM_KEY_LEN")
    }
}
