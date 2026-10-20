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

pub const TOTAL_FLASH_SIZE: usize = 2 * 1024 * 1024;
pub const VAULT_MEMORY_SIZE: usize = 64 * 1024;
pub const VAULT_FLASH_OFFSET: u32 = (TOTAL_FLASH_SIZE - VAULT_MEMORY_SIZE) as u32;
pub const XIP_BASE: u32 = 0x1000_0000;
pub const VAULT_FLASH_START: usize = XIP_BASE as usize + (VAULT_FLASH_OFFSET as usize % ERASE_PAGE_SIZE);
pub const VAULT_FLASH_END: usize = XIP_BASE as usize + TOTAL_FLASH_SIZE;
pub const VAULT_COUNT: usize = VAULT_MEMORY_SIZE / ERASE_PAGE_SIZE;

pub const fn align(addr: usize, alignment: usize) -> usize {
    (addr + alignment - 1) & !(alignment - 1)
}

pub const FIRST_VAULT_ADDR: usize = align(VAULT_FLASH_START, ERASE_PAGE_SIZE);

pub const VAULT_HEADER_CHECK_STR: &[u8] = b"Tacklebox Check";
pub const VAULT_HEADER_CHECK_LEN: usize = VAULT_HEADER_CHECK_STR.len();
pub const MAX_VAULT_NAME_LEN: usize = 206 - VAULT_HEADER_CHECK_LEN;
pub const MAX_ENTRY_TEXT_LEN: usize = 225;
