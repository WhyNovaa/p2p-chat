use serde::Serialize;
use std::io::Write;

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
