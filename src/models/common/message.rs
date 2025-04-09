use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::models::common::errors::FileError;
use crate::models::common::file::File;
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Message {
    pub data: Option<String>,
    pub file: Option<File>,
}

impl Message {
    /// Won't panic if path is None
    pub async fn build(data: Option<String>, path: Option<impl AsRef<Path> + Clone>) -> Result<Self, FileError> {

        let file = match path.is_some() {
            true => Some(File::new(path.unwrap()).await?),
            false => None,
        };

        Ok(Self {
            data,
            file
        })
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_none() && self.file.is_none()
    }
}
