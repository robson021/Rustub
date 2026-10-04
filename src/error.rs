use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),

    #[error("Failed to check file existence in the path '{0}'.")]
    CouldNotCheckFile(String),
}
