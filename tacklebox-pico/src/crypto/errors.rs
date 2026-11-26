#[derive(Debug)]
pub enum CryptoStartupError {
    I2c(embassy_rp::i2c::Error),
    IncorrectRespLen(u8),
    BadCrc(u8),
    CommandFailed(u8),
    ConfigZoneUnlocked,
    AesDisabledOnChip,
    DataZoneLockedWithoutKey,
    NoFreeSlots,
    Driver(at_cryptoauth::error::Error),
    RngFailed(at_cryptoauth::error::Error),
    SlotNotAes,
    ConfigVerifyFailed,
    DataZoneUnlocked,
}

impl CryptoStartupError {
    pub fn errno(&self) -> u8 {
        match self {
            Self::I2c(_) => 0,
            Self::IncorrectRespLen(_) => 1,
            Self::BadCrc(_) => 2,
            Self::CommandFailed(_) => 3,
            Self::ConfigZoneUnlocked => 4,
            Self::AesDisabledOnChip => 5,
            Self::DataZoneLockedWithoutKey => 6,
            Self::NoFreeSlots => 7,
            Self::Driver(_) => 8,
            Self::RngFailed(_) => 9,
            Self::SlotNotAes => 10,
            Self::ConfigVerifyFailed => 11,
            Self::DataZoneUnlocked => 12,
        }
    }
}

pub type CryptoStartupResult<T> = Result<T, CryptoStartupError>;

impl From<embassy_rp::i2c::Error> for CryptoStartupError {
    fn from(e: embassy_rp::i2c::Error) -> Self {
        Self::I2c(e)
    }
}

impl From<at_cryptoauth::error::Error> for CryptoStartupError {
    fn from(e: at_cryptoauth::error::Error) -> Self {
        Self::Driver(e)
    }
}


