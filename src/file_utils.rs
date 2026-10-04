use crate::error::FileError;
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub(crate) fn open_file(path: &str) -> Result<File, Box<dyn Error>> {
    let path = match Path::new(path).try_exists() {
        Ok(_) => Path::new(&path),
        Err(_) => {
            return Err(FileError::FileDoesNotExist(path.to_owned()).into());
        }
    };
    let file = File::open(path)?;
    Ok(file)
}
