use aes_gcm::{Aes256Gcm, Key};

use crate::storage::consts::{AES256GCM_NONCE_LEN, AES256GCM_TAG_LEN};

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
