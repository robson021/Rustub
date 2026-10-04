use crate::error::FileError;
use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(crate) fn open_file(path: &str) -> Result<BufReader<File>> {
    let path = match Path::new(path).exists() {
        true => Path::new(&path),
        false => {
            return Err(FileError::FileDoesNotExist(path.to_owned()).into());
        }
    };
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader)
}

pub(crate) fn list_all_files_with_suffix(path: &str, suffix: &str) -> Result<Vec<PathBuf>> {
    let mut files = vec![];
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|dir_entry| dir_entry.path().is_file())
        .for_each(|dir_entry| {
            let path = dir_entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                && file_name.ends_with(suffix)
            {
                files.push(path.to_owned());
            }
        });
    Ok(files)
}
