use bytemuck::{Pod, Zeroable};

use crate::storage::{consts::{AES256GCM_NONCE_LEN, AES256GCM_TAG_LEN, PBKDF2_SALT_LEN, VAULT_ENTRY_COUNT, VAULT_ITEM_SIZE, VAULT_SIZE}, crypto::Ciphertext, types::{EntryId, VaultId, VaultStatus, VaultVersion}};


#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct RawVaultHeader {
    status: u8,
    version: u8,
    _pad1: [u8; 2],
    vault_id: u32,
    salt: [u8; PBKDF2_SALT_LEN],

    // Destructured ciphertext object
    nonce: [u8; AES256GCM_NONCE_LEN],
    tag: [u8; AES256GCM_TAG_LEN],
    ciphertext: [u8; 16],

    _pad2: [u8; 188]
}
const _: () = assert!(size_of::<RawVaultHeader>() == VAULT_ITEM_SIZE);

impl RawVaultHeader {
    pub fn validate(&self) -> VaultHeader {
        let status = VaultStatus::from_u8(self.status);
        let version = VaultVersion::from_u8(self.version);
        let vault_id = VaultId::from_u32(self.vault_id);
        let salt = self.salt;
        let validator = Ciphertext::from_parts(
            self.nonce,
            self.tag,
            self.ciphertext
        );

        VaultHeader { status, version, vault_id,salt, validator }
    }
}

pub struct VaultHeader {
    status: VaultStatus,
    version: VaultVersion,
    vault_id: VaultId,
    salt: [u8; PBKDF2_SALT_LEN],
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
            _pad1: [0xFF, 2],
            vault_id: self.vault_id.id(),
            salt: self.salt,
            nonce,
            tag,
            ciphertext,
            _pad2: [0xFF; 188]
        }
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct RawVaultEntry {
    status: u8,
    version: u8,
    _pad1: [u8; 2],
    entry_id: u32,

    // Destructured ciphertext object
    nonce: [u8; AES256GCM_NONCE_LEN],
    tag: [u8; AES256GCM_TAG_LEN],
    ciphertext: [u8; 220]
}
const _: () = assert!(size_of::<RawVaultEntry>() == VAULT_ITEM_SIZE);

impl RawVaultEntry {
    pub fn validate(&self) -> VaultEntry {
        let status = VaultStatus::from_u8(self.status);
        let version = VaultVersion::from_u8(self.version);
        let entry_id = EntryId::from_u32(self.entry_id);
        let payload = Ciphertext::from_parts(
            self.nonce,
            self.tag,
            self.ciphertext
        );

        VaultEntry { status, version, entry_id, payload }
    }
}


#[repr(C)]
pub struct VaultEntry {
    status: VaultStatus,
    version: VaultVersion,
    entry_id: EntryId,
    payload: Ciphertext<220>
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
            _pad1: [0xFF; 2],
            entry_id: self.entry_id.id(),
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
