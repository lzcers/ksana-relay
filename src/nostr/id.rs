use serde::{de::Visitor, Deserialize, Serialize};
use std::fmt;

use super::Error;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Id(pub [u8; 32]);

impl Id {
    #[allow(dead_code)]
    pub fn as_hex_string(&self) -> String {
        hex::encode(&self.0)
    }
    #[allow(dead_code)]
    pub fn try_from_hex_string(v: &str) -> Result<Id, Error> {
        let vec = hex::decode(v)?;

        Ok(Id(vec
            .try_into()
            .map_err(|_| Error::WrongLengthHexString)?))
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(IdVisitor)
    }
}
struct IdVisitor;

impl Visitor<'_> for IdVisitor {
    type Value = Id;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a hexadecimal string representing 32 bytes")
    }

    fn visit_str<E>(self, v: &str) -> Result<Id, E>
    where
        E: serde::de::Error,
    {
        let vec: Vec<u8> =
            hex::decode(v).map_err(|e| serde::de::Error::custom(format!("{}", e)))?;

        Ok(Id(vec.try_into().map_err(|e: Vec<u8>| {
            E::custom(format!(
                "Id is not 32 bytes long. Was {} bytes long",
                e.len()
            ))
        })?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_id() {
        let id = Id::try_from_hex_string(
            "4d7b5695951aa06d0c429c89da21091788d0f2deb336a22eadf5545b7db3f1ed",
        )
        .unwrap();
        let id2 = Id::try_from_hex_string(
            "4d7b5695951aa06d0c429c89da21091788d0f2deb336a22eadf5545b7db3f1ed",
        )
        .unwrap();
        assert_eq!(id, id2);
        println!("{}", &id.as_hex_string())
    }
}
