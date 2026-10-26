// NOTE if this enum is updated, add new pattern to VaultVersion::from_u8
#[repr(u8)]
pub enum VaultVersion {
    Unspecified = 0xFF,
    V1 = 0xFE,
}

impl VaultVersion {
    pub fn from_u8(version: u8) -> Self {
        match version {
            0xFE => Self::V1,
            _    => Self::Unspecified
        }
    }
}


// NOTE if this enum is updated, add new pattern to VaultStatus::from_u8
#[repr(u8)]
pub enum VaultStatus {
    Unused   = 0xFF,
    Occupied = 0xA5,
    Stale    = 0x00,
    Unknown  = 0x5A,
}

impl VaultStatus {
    pub fn from_u8(status: u8) -> Self {
        match status {
            0xFF => Self::Unused,
            0xA5 => Self::Occupied,
            0x00 => Self::Stale,
            _    => Self::Unknown
        }
    }
}


#[repr(transparent)]
pub struct VaultId(u32);

#[repr(transparent)]
pub struct EntryId(u32);
