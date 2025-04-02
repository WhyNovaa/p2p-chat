#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("Couldn't read file")]
    CouldntReadFile,
    #[error("Filename is not in UTF-8")]
    WrongEncoding
}