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
    let files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|result| result.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|name| name.ends_with(suffix))
                .map(|_| entry.path().to_path_buf())
        })
        .collect();

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stub_response::StubResponse;
    use serde_json;

    #[test]
    fn opens_existing_file_and_parses_json() {
        let reader = open_file("config/default/test-stub.json").expect("open existing file");
        let stubs: Vec<StubResponse> = serde_json::from_reader(reader).expect("parse json");
        assert_eq!(stubs.len(), 2);
    }

    #[test]
    fn returns_error_for_missing_path() {
        let res = open_file("config/default/does-not-exist.json");
        assert!(res.is_err());
    }

    #[test]
    fn lists_all_files_with_suffix_returns_matching_files() {
        let files = list_all_files_with_suffix("config/default", "-stub.json").expect("list files");
        assert!(!files.is_empty());
        let mut found = false;
        for p in files {
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name == "test-stub.json" {
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "expected test-stub.json to be present");
    }
}
