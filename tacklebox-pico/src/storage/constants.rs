pub const AES_NONCE_SIZE: usize = 12;
pub const AES_TAG_SIZE: usize = 16;
pub const AES_KEY_LEN: usize = 32;
pub const PBKDF2_SALT_LEN: usize = 16;
pub const PBKDF2_ITERATIONS: u32 = 50_000;

pub const WRITE_PAGE_SIZE: usize = 256;
pub const ERASE_PAGE_SIZE: usize = 4 * 1024;
pub const WRITE_PAGES_PER_VAULT: usize = ERASE_PAGE_SIZE / WRITE_PAGE_SIZE;
pub const HEADERS_PER_VAULT: usize = 1;
pub const ENTRIES_PER_VAULT: usize = WRITE_PAGES_PER_VAULT - HEADERS_PER_VAULT;

pub const VAULT_HEADER_CHECK_STR: &[u8] = b"Tacklebox Check";
pub const VAULT_HEADER_CHECK_LEN: usize = VAULT_HEADER_CHECK_STR.len();
pub const MAX_VAULT_NAME_LEN: usize = 206 - VAULT_HEADER_CHECK_LEN;
pub const MAX_ENTRY_TEXT_LEN: usize = 225;
