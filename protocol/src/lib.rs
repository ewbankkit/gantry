#[macro_use]
extern crate serde_derive;

extern crate rmp_serde as rmps;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

pub fn serialize<T>(item: T) -> ::std::result::Result<Vec<u8>, Box<dyn ::std::error::Error>>
where
    T: Serialize,
{
    let mut buf = Vec::new();
    item.serialize(&mut Serializer::new(&mut buf))?;
    Ok(buf)
}

pub fn deserialize<'de, T: Deserialize<'de>>(
    buf: &[u8],
) -> ::std::result::Result<T, Box<dyn ::std::error::Error>> {
    let mut de = Deserializer::new(Cursor::new(buf));
    match Deserialize::deserialize(&mut de) {
        Ok(t) => Ok(t),
        Err(e) => Err(format!("Failed to de-serialize: {}", e).into()),
    }
}

pub mod catalog;
pub mod stream;

pub mod token {
    pub enum TokenType {
        Actor,
        Account,
        Operator,
    }
}
