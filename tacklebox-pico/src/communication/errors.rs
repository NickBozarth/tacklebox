use embassy_usb::driver::EndpointError;


#[derive(Debug, PartialEq)]
pub enum CommunicationError {
    Timeout,
    BufferOverflowError,
    DeviceDisabled,
}

impl From<EndpointError> for CommunicationError {
    fn from(value: EndpointError) -> Self {
        match value {
            EndpointError::Disabled => Self::DeviceDisabled,
            EndpointError::BufferOverflow => Self::BufferOverflowError
        }
    }
}


pub type CommunicationResult<T> = core::result::Result<T, CommunicationError>;


#[derive(Debug, PartialEq)]
pub enum ConnectionError {
    FlashFetchError,
    SerialParseError,
    TaskSpawnError,
    IoTaskSpawnError,
}

pub type ConnectionResult<T> = core::result::Result<T, ConnectionError>;
