use crate::error::FileError;
use anyhow::Result;
use std::fs::File;
use std::path::Path;

pub(crate) fn open_file(path: &str) -> Result<File> {
    let path = match Path::new(path).exists() {
        true => Path::new(&path),
        false => {
            return Err(FileError::FileDoesNotExist(path.to_owned()).into());
        }
    };
    let file = File::open(path)?;
    Ok(file)
}
