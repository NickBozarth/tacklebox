pub const PBKDF2_SALT_LEN: usize = 16;
pub const PBKDF2_ITERATIONS: u32 = 50_000;
pub const AES256GCM_NONCE_LEN: usize = 12;
pub const AES256GCM_TAG_LEN: usize = 16;
pub const AES256GCM_KEY_LEN: usize = 32;

pub const SEED_SAMPLES: usize = 512;

// STORAGE CONSTANTS
pub const FLASH_WRITE_PAGE_SIZE: usize = 256;
pub const FLASH_ERASE_PAGE_SIZE: usize = 4096;

pub const VAULT_ITEM_SIZE: usize = FLASH_WRITE_PAGE_SIZE;
pub const VAULT_SIZE: usize = FLASH_ERASE_PAGE_SIZE;
pub const VAULT_ITEM_COUNT: usize = VAULT_SIZE / VAULT_ITEM_SIZE;
pub const VAULT_ENTRY_COUNT: usize = VAULT_ITEM_COUNT - 1;

pub const MAX_STRING_LEN: usize = 110;
