#[derive(Debug, PartialEq, Clone)]
pub enum MessageError {
    SerializeError(postcard::Error)
}
pub type MessageResult<T> = Result<T, MessageError>;


#[derive(Debug, PartialEq, Clone)]
pub enum MessageReaderError {
    BufferOverflow,
    DeserError,
}
pub type MessageReaderResult<T> = Result<T, MessageReaderError>;


#[derive(Debug, PartialEq, Clone)]
pub enum MessageWriterError {
    SerializeError(MessageError)
}
pub type MessageWriterResult<T> = Result<T, MessageWriterError>;
