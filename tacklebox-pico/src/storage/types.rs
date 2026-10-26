// NOTE if this enum is updated, add new pattern to VaultVersion::from_u8
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
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


#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct VaultId(u32);

impl VaultId {
    pub const INVALID: Self = Self(u32::MAX);

    pub const fn id(&self) -> u32 {
        self.0
    }

    pub const fn from_u32(id: u32) -> Self {
        // TODO bounds check
        // if id < SOME_VAL {
        //     Self(id)
        // } else {
        //     Self::INVALID
        // }

        Self(id)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct EntryId(u32);

impl EntryId {
    pub const INVALID: Self = Self(u32::MAX);

    pub const fn id(&self) -> u32 {
        self.0
    }

    pub const fn from_u32(id: u32) -> Self {
        // TODO bounds check
        // if id < SOME_VAL {
        //     Self(id)
        // } else {
        //     Self::INVALID
        // }

        Self(id)
    }
}
