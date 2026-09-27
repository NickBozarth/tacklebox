use core::{error::Error, fmt::{Display}};

pub type CommunicationResult<T> = core::result::Result<T, CommunicationError>;

#[derive(Debug, PartialEq)]
pub enum CommunicationError {
    DeviceDisabled
}

impl Display for CommunicationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DeviceDisabled => write!(f, "Attempted to send data to a disabled device")
        }
    }
}

impl Error for CommunicationError {}




pub type ConnectionResult<T> = core::result::Result<T, ConnectionError>;

#[derive(Debug, PartialEq)]
pub enum ConnectionError {
    IoTaskSpawnError,
    FlashFetchError,
    SerialParseError,
    TaskSpawnError
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IoTaskSpawnError => write!(f, "Failed to spawn IO task"),
            Self::FlashFetchError => write!(f, "Failed to fetch unique id from flash"),
            Self::SerialParseError => write!(f, "Failed to parse data into serial no."),
            Self::TaskSpawnError => write!(f, "Failed to spawn connection task")
        }
    }
}

impl Error for ConnectionError {}
