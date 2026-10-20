pub enum CryptoError {
    DecryptionError,
    EncryptionError,
}

pub type CryptoResult<T> = Result<T, CryptoError>;
