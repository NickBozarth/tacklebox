use bytemuck::{Pod, Zeroable};

use crate::storage::{consts::{AES256GCM_NONCE_LEN, AES256GCM_TAG_LEN, PBKDF2_SALT_LEN, VAULT_ENTRY_COUNT, VAULT_ITEM_SIZE, VAULT_SIZE}, types::{EntryId, VaultId, VaultStatus, VaultVersion}};


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

#[repr(C)]
pub struct VaultHeader {
    status: VaultStatus,
    version: VaultVersion,
    _pad1: [u8; 2],
    vault_id: VaultId,
    salt: [u8; PBKDF2_SALT_LEN],

    // TODO wrap this into a ciphertext struct
    nonce: [u8; AES256GCM_NONCE_LEN],
    tag: [u8; AES256GCM_TAG_LEN],
    validator_ciphertext: [u8; 16],

    _pad2: [u8; 188]
}

const _: () = assert!(size_of::<RawVaultHeader>() == size_of::<VaultHeader>());
const _: () = assert!(size_of::<VaultHeader>() == VAULT_ITEM_SIZE);



#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct RawVaultEntry {
    status: u8,
    version: u8,
    _pad1: [u8; 2],
    entry_id: u32,

    nonce: [u8; AES256GCM_NONCE_LEN],
    tag: [u8; AES256GCM_TAG_LEN],
    ciphertext: [u8; 220]
}


#[repr(C)]
pub struct VaultEntry {
    status: VaultStatus,
    version: VaultVersion,
    entry_id: EntryId,

    // TODO wrap this into a ciphertext struct
    nonce: [u8; AES256GCM_NONCE_LEN],
    tag: [u8; AES256GCM_TAG_LEN],
    ciphertext: [u8; 220]
}

const _: () = assert!(size_of::<RawVaultEntry>() == size_of::<VaultEntry>());
const _: () = assert!(size_of::<VaultEntry>() == VAULT_ITEM_SIZE);



pub struct Vault {
    header: VaultHeader,
    entries: [VaultEntry; VAULT_ENTRY_COUNT],
}

const _: () = assert!(size_of::<Vault>() == VAULT_SIZE);
