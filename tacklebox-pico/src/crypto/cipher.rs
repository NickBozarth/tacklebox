use core::marker::PhantomData;

use at_cryptoauth::{der::zeroize::Zeroizing, memory::Slot, signature::rand_core::RngCore};
use tacklebox_core::comms::consts::{BASE_NONCE_LEN, BLOCK_SIZE};

use crate::crypto::startup::AtCaClientAsync;

// ATECC608B accepts 16 byte chunks at a time
const DIGEST_BLOCK_SIZE: usize = 16;
const DIGEST_BLOCK_COUNT: usize = BLOCK_SIZE / DIGEST_BLOCK_SIZE;
const _: () = assert!(
    BLOCK_SIZE % DIGEST_BLOCK_SIZE == 0,
    "BLOCK_SIZE must be a multiple of DIGEST_BLOCK_SIZE for block encryption to work"
);
const _: () = assert!(
    BASE_NONCE_LEN + 4 == DIGEST_BLOCK_SIZE,
    "BASE_NONCE_LEN must leave room for a 4-byte block counter"
);
type DigestBlocks = Zeroizing<[[u8; DIGEST_BLOCK_SIZE]; DIGEST_BLOCK_COUNT]>;
type NonceBuf = Zeroizing<[u8; BASE_NONCE_LEN]>;


pub enum CipherError {
    EncryptError,
    DecryptError,
}
pub type CipherResult<T> = Result<T, CipherError>;


// Marker types to denote direction of cipher
pub struct Encrypt;
pub struct Decrypt;

pub struct Cipher<DIR> {
    digest_blocks: DigestBlocks,
    base_nonce: Option<NonceBuf>,
    block_idx: u32,

    _cipher_direction: PhantomData<DIR>
}


fn digest_blocks_from_block(src: &[u8; BLOCK_SIZE]) -> DigestBlocks {
    // BLOCK_SIZE == DIGEST_BLOCK_SIZE * DIGEST_BLOCK_COUNT
    let dst: [[u8; DIGEST_BLOCK_SIZE]; DIGEST_BLOCK_COUNT] = core::array::from_fn(|i| {
        let start = i * DIGEST_BLOCK_SIZE;
        src[start..start + DIGEST_BLOCK_SIZE].try_into().unwrap()
    });

    Zeroizing::new(dst)
}

fn block_from_digest_blocks(src: &DigestBlocks) -> Zeroizing<[u8; BLOCK_SIZE]> {
    let mut dst = [0u8; BLOCK_SIZE];
    for (i, block) in src.iter().enumerate() {
        let start = i * DIGEST_BLOCK_SIZE;
        dst[start..start + DIGEST_BLOCK_SIZE].copy_from_slice(block);
    }

    dst.into()
}

impl<T> Cipher<T> {
    fn new(
        buf_ref: &[u8; 128],
        base_nonce_ref: Option<&[u8; BASE_NONCE_LEN]>,
        block_idx: u32
    ) -> Self {
        let digest_blocks = digest_blocks_from_block(buf_ref);
        let base_nonce = match base_nonce_ref {
            Some(nonce) => { Some(Zeroizing::new(*nonce)) },
            None => None
        };

        Self {
            digest_blocks,
            base_nonce,
            block_idx,
            _cipher_direction: PhantomData
        }
    }

    async fn run_ctr(
        mut self,
        atca: &AtCaClientAsync<'_>,
        slot: Slot,
        base_nonce: &[u8; BASE_NONCE_LEN],
        on_err: fn() -> CipherError
    ) -> CipherResult<Zeroizing<[u8; BLOCK_SIZE]>> {
        let mut aes = atca.aes(slot);

        let mut cipher_blocks = [[0u8; DIGEST_BLOCK_SIZE]; DIGEST_BLOCK_COUNT];
        let mut block_counter = self.block_idx.wrapping_mul(8);

        for (block, cipher) in self.digest_blocks.iter_mut().zip(cipher_blocks.iter_mut()) {
            let block_counter_bytes = block_counter.to_le_bytes();
            let mut counter_block: [u8; DIGEST_BLOCK_SIZE] = [0u8; DIGEST_BLOCK_SIZE];
            counter_block[..BASE_NONCE_LEN].copy_from_slice(base_nonce.as_ref());
            counter_block[DIGEST_BLOCK_SIZE - 4..].copy_from_slice(&block_counter_bytes);

            aes.encrypt(&counter_block, cipher)
                .await
                .map_err(|_| on_err())?;

            for (b, c) in block.iter_mut().zip(cipher) {
                *b ^= *c;
            }

            block_counter = block_counter.wrapping_add(1);
        }

        Ok(block_from_digest_blocks(&self.digest_blocks))
    }
}

impl Cipher<Encrypt> {
    pub fn encrypt_cipher(
        buf_ref: &[u8; 128],
        base_nonce_ref: Option<&[u8; BASE_NONCE_LEN]>,
        block_idx: u32
    ) -> Self {
        Self::new(buf_ref, base_nonce_ref, block_idx)
    }

    // Cipher<Encrypt> may not be provided a base_nonce so one must be generated
    async fn base_nonce(&self, atca: &AtCaClientAsync<'_>) -> Zeroizing<[u8; BASE_NONCE_LEN]> {
        match &self.base_nonce {
            Some(nonce) => nonce.clone(),
            None => {
                let mut base_nonce = [0u8; BASE_NONCE_LEN];
                atca.random()
                    .fill_bytes(&mut base_nonce);
                base_nonce.into()
            }
        }
    }

    pub async fn encrypt(
        self,
        atca: &AtCaClientAsync<'_>,
        slot: Slot
    ) -> CipherResult<([u8; BLOCK_SIZE], [u8; BASE_NONCE_LEN])> {
        let base_nonce = self.base_nonce(atca).await;

        self.run_ctr(
            atca,
            slot,
            &base_nonce,
            || CipherError::EncryptError
        )
            .await
            .map(|buf| (*buf, *base_nonce))
    }
}

impl Cipher<Decrypt> {
    pub fn decrypt_cipher(
        buf_ref: &[u8; 128],
        base_nonce_ref: &[u8; BASE_NONCE_LEN],
        block_idx: u32
    ) -> Self {
        Self::new(buf_ref, Some(base_nonce_ref), block_idx)
    }

    // Cipher<Decrypt>.base_nonce will always contain Some(base_nonce)
    fn base_nonce(&self) -> Zeroizing<[u8; BASE_NONCE_LEN]> {
        self.base_nonce.clone().unwrap()
    }

    pub async fn decrypt(
        self,
        atca: &AtCaClientAsync<'_>,
        slot: Slot
    ) -> CipherResult<Zeroizing<[u8; BLOCK_SIZE]>> {
        let nonce = self.base_nonce();
        self.run_ctr(
            atca,
            slot,
            &nonce,
            || CipherError::DecryptError
        )
            .await
    }
}
