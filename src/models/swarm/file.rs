use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::models::swarm::errors::FileError;

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub data: Vec<u8>,
}

impl File {
    pub async fn new(path: impl AsRef<Path> + Clone) -> Result<Self, FileError> {
        let data = tokio::fs::read(path.clone()).await.map_err(|_| FileError::CouldntReadFile)?;

        let name = path.as_ref()
            .file_name()
            .unwrap() // can unwrap because if path doesn't provide name we can't read file
            .to_str()
            .ok_or(FileError::WrongEncoding)?
            .to_owned();

        Ok(Self { name, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use tokio::fs;

    #[tokio::test]
    async fn test_create_file_success() {
        let mut file = NamedTempFile::new().unwrap();
        let path = file.path().to_owned();

        let test_data = b"Hello world";
        file.write_all(test_data).unwrap();

        let result = File::new(&path).await.unwrap();

        assert_eq!(result.name, path.file_name().unwrap().to_str().unwrap());
        assert_eq!(result.data, test_data);
    }

    #[tokio::test]
    async fn test_nonexistent_file() {
        let path = "/non/existent/path.txt";
        let result = File::new(path).await;
        assert!(matches!(result, Err(FileError::CouldntReadFile)));
    }

    #[tokio::test]
    async fn test_empty_filename() {
        let dir = tempfile::tempdir().unwrap();
        let result = File::new(dir.path()).await;
        assert!(matches!(result, Err(FileError::CouldntReadFile)));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_invalid_encoding() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let invalid_bytes = &[0x80, 0x81];
        let invalid_os_str = OsStr::from_bytes(invalid_bytes);
        let path = Path::new(invalid_os_str);

        let _ = fs::File::create(&path).await.unwrap();

        let result = File::new(&path).await;
        assert!(matches!(result, Err(FileError::WrongEncoding)));

        fs::remove_file(&path).await.unwrap();
    }
}