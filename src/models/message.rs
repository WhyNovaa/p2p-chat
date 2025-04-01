use std::io::{Read, Write};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub data: String,
    timestamp: Duration,
}

impl Message {
    pub fn new(data: String) -> Self {
        let timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
        Self {
            data,
            timestamp,
        }
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