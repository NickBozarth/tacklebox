use aes_gcm::{Aes256Gcm, Key};
use bytemuck::{Pod, Zeroable};

use crate::storage::{consts::{AES256GCM_NONCE_LEN, AES256GCM_TAG_LEN, MAX_STRING_LEN, PBKDF2_SALT_LEN, VAULT_ENTRY_COUNT, VAULT_ITEM_SIZE, VAULT_SIZE}, crypto::Ciphertext, types::{EntryId, TruncatingBuf, VaultId, VaultStatus, VaultVersion}};


#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct RawVaultHeader {
    status: u8,
    version: u8,
    name_len: u8,
    _pad1: u8,
    vault_id: u32,
    salt: [u8; PBKDF2_SALT_LEN],
    name: [u8; MAX_STRING_LEN],

    // Destructured ciphertext object
    nonce: [u8; AES256GCM_NONCE_LEN],
    tag: [u8; AES256GCM_TAG_LEN],
    ciphertext: [u8; 16],

    _pad2: [u8; 78]
}
const _: () = assert!(size_of::<RawVaultHeader>() == VAULT_ITEM_SIZE);

impl RawVaultHeader {
    pub fn validate(&self) -> VaultHeader {
        let status = VaultStatus::from_u8(self.status);
        let version = VaultVersion::from_u8(self.version);
        let vault_id = VaultId::from_u32(self.vault_id);
        let salt = self.salt;
        let name = self.name;
        let name_len = self.name_len;
        let validator = Ciphertext::from_parts(
            self.nonce,
            self.tag,
            self.ciphertext
        );

        VaultHeader { status, version, vault_id, salt, name, name_len, validator }
    }
}

pub struct VaultHeader {
    status: VaultStatus,
    version: VaultVersion,
    vault_id: VaultId,
    salt: [u8; PBKDF2_SALT_LEN],
    name: [u8; MAX_STRING_LEN],
    name_len: u8,
    validator: Ciphertext<16>,
}

impl VaultHeader {
    pub fn as_raw(&self) -> RawVaultHeader {
        let (
            nonce,
            tag,
            ciphertext
        ) = self.validator.as_parts();

        RawVaultHeader {
            status: self.status as u8,
            version: self.version as u8,
            name_len: self.name_len,
            _pad1: 0xFF,
            vault_id: self.vault_id.id(),
            name: self.name,
            salt: self.salt,
            nonce,
            tag,
            ciphertext,
            _pad2: [0xFF; 78]
        }
    }

    pub fn new(passphrase: &[u8], name_buf: TruncatingBuf<'_>) -> Self {
        let mut name = [0u8; 188];
        let name_len = name_buf.copy_truncating(&mut name);

        todo!()
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct RawVaultEntry {
    status: u8,
    version: u8,
    name_len: u8,
    _pad1: u8,
    entry_id: u32,
    name: [u8; MAX_STRING_LEN],

    // Destructured ciphertext object
    nonce: [u8; AES256GCM_NONCE_LEN],
    tag: [u8; AES256GCM_TAG_LEN],
    ciphertext: [u8; MAX_STRING_LEN]
}
const _: () = assert!(size_of::<RawVaultEntry>() == VAULT_ITEM_SIZE);

impl RawVaultEntry {
    pub fn validate(&self) -> VaultEntry {
        let status = VaultStatus::from_u8(self.status);
        let version = VaultVersion::from_u8(self.version);
        let entry_id = EntryId::from_u32(self.entry_id);
        let name = self.name;
        let name_len = self.name_len;
        let payload = Ciphertext::from_parts(
            self.nonce,
            self.tag,
            self.ciphertext
        );

        VaultEntry { status, version, entry_id, name, name_len, payload }
    }
}


#[repr(C)]
pub struct VaultEntry {
    status: VaultStatus,
    version: VaultVersion,
    entry_id: EntryId,
    name: [u8; MAX_STRING_LEN],
    name_len: u8,
    payload: Ciphertext<MAX_STRING_LEN>,
}

impl VaultEntry {
    pub fn as_raw(&self) -> RawVaultEntry {
        let (
            nonce,
            tag,
            ciphertext
        ) = self.payload.as_parts();

        RawVaultEntry {
            status: self.status as u8,
            version: self.version as u8,
            name_len: self.name_len,
            _pad1: 0xFF,
            entry_id: self.entry_id.id(),
            name: self.name,
            nonce,
            tag,
            ciphertext
        }
    }
}


pub struct RawVault {
    header: RawVaultHeader,
    entries: [RawVaultEntry; VAULT_ENTRY_COUNT]
}
const _: () = assert!(size_of::<RawVault>() == VAULT_SIZE);

pub struct Vault {
    header: VaultHeader,
    entries: [VaultEntry; VAULT_ENTRY_COUNT],
}
