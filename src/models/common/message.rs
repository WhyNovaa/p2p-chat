use crate::models::common::file::File;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Message {
    pub data: Option<String>,
    pub file: Option<File>,
}

impl Message {
    /// Won't panic if path is None
    pub async fn build(data: Option<String>, file: Option<File>) -> Self {
        Self { data, file }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_none() && self.file.is_none()
    }

    pub async fn try_to_download_file(self, path: String) -> bool {
        if let Some(file) = self.file {
            return file.save(path).await;
        }
        false
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (&self.data, &self.file) {
            (Some(data), Some(file)) => {
                write!(f, "{}, File: {}", data, file.name)
            }
            (Some(data), None) => {
                write!(f, "{}", data)
            }
            (None, Some(file)) => {
                write!(f, "File: {}", file.name)
            }
            (None, None) => {
                write!(f, "*Empty message*")
            }
        }
    }
}
