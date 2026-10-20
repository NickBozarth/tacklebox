/*
 * status:                      1  byte
 * vault_id:                    1  byte
 * version:                     2  bytes
 * salt:                        16 bytes
 * passphrase_check_ciphertext: Ciphertext<VAULT_HEADER_CHECK_LEN>
 *                              (29 + VAULT_HEADER_CHECK_LEN)
 * vault_name:                  VAULT_NAME_LEN
 *                              (206 - VAULT_HEADER_CHECK_LEN)
 * vault_name_len:              1 byte
 * -----------------------------------------------------------------------------------
 * 1 + 1 + 2 + 16 + (29 + VAULT_HEADER_CHECK_LEN) + (206 - VAULT_HEADER_CHECK_LEN) + 1
 * = 256
 */
#[repr(C, align(256))]
pub struct VaultHeader {
    status: VaultStatus,
    vault_id: u8,
    version: VaultVersion,
    salt: [u8; PBKDF2_SALT_LEN],
    passphrase_check_ciphertext: Ciphertext<VAULT_HEADER_CHECK_LEN>,
    vault_name: [u8; VAULT_NAME_LEN],
    vault_name_len: u8,
}
