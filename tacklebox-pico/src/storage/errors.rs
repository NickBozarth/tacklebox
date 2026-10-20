#[derive(Debug, PartialEq, Clone)]
pub enum CryptoError {
    DecryptionError,
    EncryptionError,
}
pub type CryptoResult<T> = Result<T, CryptoError>;


#[derive(Debug, PartialEq, Clone)]
pub enum WritePageError {
    InvalidAlignment,
    InvalidLength,
}
pub type WritePageResult<T> = Result<T, WritePageError>;


#[derive(Debug, PartialEq, Clone)]
pub enum VaultError {
    CryptoError(CryptoError),
    EraseNecessary,
    FlashWriteError,
    InvalidEntry,
    InvalidHeader,
    InvalidPageIndex,
    InvalidPassphrase,
    InvalidVaultIndex,
}
pub type VaultResult<T> = Result<T, VaultError>;
