#[derive(Debug, PartialEq, Clone)]
pub enum MessageError {
    SerializeError(postcard::Error)
}
pub type MessageResult<T> = Result<T, MessageError>;


#[derive(Debug, PartialEq, Clone)]
pub enum MessageReaderError {
    BufferOverflow,
    DeserError,
    Disconnected,
    Empty,
}
pub type MessageReaderResult<T> = Result<T, MessageReaderError>;


#[derive(Debug, PartialEq, Clone)]
pub enum MessageWriterError {
    Disconnected,
    SerializeError(MessageError),
}
pub type MessageWriterResult<T> = Result<T, MessageWriterError>;
