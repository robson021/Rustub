use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),
}
