use core::{error::Error, fmt::{Display}};

use embassy_usb::driver::EndpointError;


pub type CommunicationResult<T> = core::result::Result<T, CommunicationError>;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum CommunicationError {
    BufferOverflow = 0,
    DeviceDisabled
}

impl Display for CommunicationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BufferOverflow => write!(f, "BufferOverflow when writing data"),
            Self::DeviceDisabled => write!(f, "Attempted to send data to a disabled device")
        }
    }
}

impl Error for CommunicationError {}

impl From<EndpointError> for CommunicationError {
    fn from(value: EndpointError) -> Self {
        match value {
            EndpointError::BufferOverflow => Self::BufferOverflow,
            EndpointError::Disabled => Self::DeviceDisabled
        }
    }
}




pub type ConnectionResult<T> = core::result::Result<T, ConnectionError>;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ConnectionError {
    IoTaskSpawnError = 0,
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
