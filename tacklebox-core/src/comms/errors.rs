#[derive(Debug, PartialEq, Clone)]
pub enum MessageError {
    SerializeError(postcard::Error)
}

pub type MessageResult<T> = Result<T, MessageError>;
