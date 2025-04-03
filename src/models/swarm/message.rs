use std::io::{Read, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use crate::models::swarm::errors::FileError;
use crate::models::swarm::file::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub data: Option<String>,
    pub file: Option<File>,
}

impl Message {
    // Won't panic if path is None
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

pub trait Encode: Sized {
    type Error;

    fn encode(&self, writer: &mut impl Write) -> Result<(), Self::Error>;

    fn encode_to_vec(&self) -> Result<Vec<u8>, Self::Error> {
        let mut buff = Vec::new();
        self.encode(&mut buff)?;
        Ok(buff)
    }
}

impl<T: Serialize> Encode for T {
    type Error = bincode::error::EncodeError;

    fn encode(&self, writer: &mut impl Write) -> Result<(), Self::Error> {
        bincode::serde::encode_into_std_write(self, writer, bincode::config::standard())?;
        Ok(())
    }
}
pub trait Decode: Sized {
    type Error;

    fn decode(reader: &mut impl Read) -> Result<Self, Self::Error>;
}


impl<T: DeserializeOwned> Decode for T {
    type Error = bincode::error::DecodeError ;

    fn decode(reader: &mut impl Read) -> Result<Self, Self::Error> {
        bincode::serde::decode_from_std_read(reader, bincode::config::standard())
    }
}