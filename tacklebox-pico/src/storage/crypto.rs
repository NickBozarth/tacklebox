use aes_gcm::{AeadInOut, Aes256Gcm, Key, KeyInit, Nonce, Tag, aead::consts::U12, aes::cipher::InOutBuf};
use pbkdf2::{pbkdf2_hmac, sha2::Sha256};

use crate::storage::{consts::{AAD_LEN, AES256GCM_KEY_LEN, AES256GCM_NONCE_LEN, AES256GCM_TAG_LEN, PBKDF2_ITERATIONS, PBKDF2_SALT_LEN}, rng::rng_fill_bytes, types::{TruncatingBuf, ZeroizingBuf}};

pub struct Ciphertext<const N: usize> {
    nonce: Nonce<U12>,
    tag: Tag,
    ciphertext: [u8; N],
    ciphertext_len: u8
}

impl<const N: usize> Ciphertext<N> {
    pub fn from_parts(
        nonce: [u8; AES256GCM_NONCE_LEN],
        tag: [u8; AES256GCM_TAG_LEN],
        ciphertext: [u8; N],
        ciphertext_len: u8,
    ) -> Self {
        let nonce = Nonce::try_from(nonce)
            .expect("Nonce with size AES256GCM_NONCE_LEN will never fail");
        let tag = Tag::try_from(tag)
            .expect("Tag with size AES256GCM_TAG_LEN will never fail");

        Self { nonce, tag, ciphertext, ciphertext_len }
    }

    pub fn as_parts(&self) -> ([u8; AES256GCM_NONCE_LEN], [u8; AES256GCM_TAG_LEN], [u8; N], u8) {
        let mut nonce = [0u8; AES256GCM_NONCE_LEN];
        nonce.copy_from_slice(self.nonce.as_slice());
        let mut tag = [0u8; AES256GCM_TAG_LEN];
        tag.copy_from_slice(self.tag.as_slice());

        (nonce, tag, self.ciphertext, self.ciphertext_len)
    }


    pub fn encrypt(
        plaintext: TruncatingBuf<'_>,
        key_bytes: &KeyBytes
    ) -> Result<Self, aes_gcm::Error> {
        let key = key_bytes.as_key();
        let aad = &key_bytes.aad;
        let cipher = Aes256Gcm::new(&key);
        let nonce: Nonce<U12> = {
            let mut nonce_bytes = [0u8; AES256GCM_NONCE_LEN];
            rng_fill_bytes(&mut nonce_bytes);
            nonce_bytes
                .as_slice()
                .try_into()
                .expect("Only fails if len != AES256GCM_NONCE_LEN, which will never happen")
        };
        let mut ciphertext = [0u8; N];
        let ciphertext_len = plaintext.copy_to_buf_truncating(&mut ciphertext);

        let tag = cipher
            .encrypt_inout_detached(
                &nonce,
                aad,
                InOutBuf::from(&mut ciphertext[..ciphertext_len])
            )?;

        Ok(Ciphertext {
            nonce,
            tag,
            ciphertext,
            ciphertext_len: ciphertext_len as u8
        })
    }

    pub fn decrypt(
        &self,
        key_bytes: &KeyBytes
    ) -> Result<ZeroizingBuf<N>, aes_gcm::Error> {
        let key = key_bytes.as_key();
        let cipher = Aes256Gcm::new(&key);
        let aad = &key_bytes.aad;
        let mut out_buf = [0u8; N];
        let out_len = self.ciphertext_len as usize;
        out_buf[..out_len]
            .copy_from_slice(&self.ciphertext[..out_len]);

        cipher
            .decrypt_inout_detached(
                &self.nonce,
                aad,
                InOutBuf::from(&mut out_buf[..out_len]),
                &self.tag
            )
            .map(|_| ZeroizingBuf::new(out_buf, out_len))
    }
}

pub struct KeyBytes {
    key_bytes: [u8; AES256GCM_KEY_LEN],
    aad: [u8; AAD_LEN]
}

impl KeyBytes {
    pub fn new(passphrase: &[u8], salt: &[u8; PBKDF2_SALT_LEN], nonce: &[u8; AES256GCM_NONCE_LEN]) -> Self {
        let mut key_bytes = [0u8; AES256GCM_KEY_LEN];
        pbkdf2_hmac::<Sha256>(
            passphrase,
            salt,
            PBKDF2_ITERATIONS,
            &mut key_bytes
        );

        let mut aad = [0u8; AAD_LEN];
        aad[..PBKDF2_SALT_LEN].copy_from_slice(salt);
        aad[PBKDF2_SALT_LEN..].copy_from_slice(nonce);

        Self { key_bytes, aad }
    }

    pub fn new_no_aad(passphrase: &[u8], salt: &[u8; PBKDF2_SALT_LEN]) -> Self {
        Self::new(passphrase, salt, &[0u8; 12])
    }

    fn as_key(&self) -> Key<Aes256Gcm> {
        self.key_bytes
            .as_slice()
            .try_into()
            .expect("Only fails if self.0.len() != AES256GCM_KEY_LEN")
    }
}
