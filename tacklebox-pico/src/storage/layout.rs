use crate::storage::{constants::{ENTRIES_PER_VAULT, ERASE_PAGE_SIZE, MAX_ENTRY_TEXT_LEN, MAX_VAULT_NAME_LEN, PBKDF2_SALT_LEN, VAULT_HEADER_CHECK_LEN, VAULT_HEADER_CHECK_STR, WRITE_PAGE_SIZE}, crypto::{Ciphertext, OwnedPlaintext, Passphrase, PlaintextBuffer, StaticPlaintext, UnbuiltPassphrase}, errors::{VaultError, VaultResult}, types::{AsWritePage, FromWritePage, VaultStatus, VaultVersion}};

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
    vault_name: [u8; MAX_VAULT_NAME_LEN],
    vault_name_len: u8,
}

impl VaultHeader {
    pub fn new(
        unbuilt_passphrase: UnbuiltPassphrase<'_>,
        vault_id: u8,
        vault_name: &[u8]
    ) -> VaultResult<Self> {
        let (passphrase, salt) = unbuilt_passphrase.build();
        let passphrase_check_plaintext = StaticPlaintext::new(VAULT_HEADER_CHECK_STR);
        let passphrase_check_ciphertext = Ciphertext::new(
            passphrase_check_plaintext,
            passphrase
        )
        .map_err(VaultError::CryptoError)?;


        let vault_name_len = usize::min(vault_name.len(), MAX_VAULT_NAME_LEN);
        let vault_name_len = usize::min(vault_name_len, u8::MAX as usize);
        let mut vault_name_buf = [0u8; MAX_VAULT_NAME_LEN];
        vault_name_buf[..vault_name_len].copy_from_slice(&vault_name[..vault_name_len]);

        Ok(Self {
            status: VaultStatus::Valid,
            vault_id,
            version: VaultVersion::V1,
            salt,
            passphrase_check_ciphertext,
            vault_name: vault_name_buf,
            vault_name_len: vault_name_len as u8
        })
    }

    pub fn validate_and_build_passphrase(&self, unbuilt_passphrase: UnbuiltPassphrase<'_>) -> VaultResult<Passphrase> {
        if self.status != VaultStatus::Valid {
            return Err(VaultError::InvalidHeader);
        }

        let passphrase = unbuilt_passphrase.build_with_salt(self.salt);
        let pass_is_valid = self.passphrase_check_ciphertext
            .decrypt(&passphrase)
            .is_ok_and(|plaintext| plaintext.as_slice() == VAULT_HEADER_CHECK_STR);

        if pass_is_valid {
            Ok(passphrase)
        } else {
            Err(VaultError::InvalidPassphrase)
        }
    }
}

const _: () = assert!(size_of::<VaultHeader>()  == WRITE_PAGE_SIZE);
const _: () = assert!(align_of::<VaultHeader>() == WRITE_PAGE_SIZE);
// SAFETY: size and alignment checked above
unsafe impl AsWritePage for VaultHeader {}
// SAFETY: size and alignment checked above
unsafe impl FromWritePage for VaultHeader {}


/*
 * status:      1 byte
 * _pad_        1 byte
 * ciphertext:  29 + MAX_ENTRY_TEXT_LEN
 * -----------------------------------
 * 1 + 1 + (29 + MAX_ENTRY_TEXT_LEN) = 256
 * ∴ MAX_ENTRY_TEXT_LEN = 225
 */
#[repr(C, align(256))]
pub struct VaultEntry {
    status: VaultStatus,
    ciphertext: Ciphertext<MAX_ENTRY_TEXT_LEN>,
}

impl VaultEntry {
    pub fn unlock(&self, passphrase: &Passphrase) -> VaultResult<OwnedPlaintext<MAX_ENTRY_TEXT_LEN>> {
        if self.status != VaultStatus::Valid {
            return Err(VaultError::InvalidHeader);
        }

        self.ciphertext
            .decrypt(passphrase)
            .map_err(VaultError::CryptoError)
    }
}

const _: () = assert!(size_of::<VaultEntry>()  == WRITE_PAGE_SIZE);
const _: () = assert!(align_of::<VaultEntry>() == WRITE_PAGE_SIZE);
// SAFETY: size and alignment checked above
unsafe impl AsWritePage for VaultEntry {}
// SAFETY: size and alignment checked above
unsafe impl FromWritePage for VaultEntry {}


#[repr(C, align(4096))]
pub struct Vault {
    header: VaultHeader,
    entries: [VaultEntry; ENTRIES_PER_VAULT]
}

const _: () = assert!(size_of::<Vault>()  == ERASE_PAGE_SIZE);
const _: () = assert!(align_of::<Vault>() == ERASE_PAGE_SIZE);
