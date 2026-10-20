pub const AES_NONCE_SIZE: usize = 12;
pub const AES_TAG_SIZE: usize = 16;
pub const AES_KEY_LEN: usize = 32;
pub const PBKDF2_SALT_LEN: usize = 16;
pub const PBKDF2_ITERATIONS: u32 = 50_000;

pub const WRITE_PAGE_SIZE: usize = 256;
pub const ERASE_PAGE_SIZE: usize = 4 * 1024;
