use std::io::Read;
use serde::de::DeserializeOwned;

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