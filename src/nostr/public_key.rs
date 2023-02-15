use serde::{de::Visitor, Deserialize, Serialize, Serializer};

use super::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey(pub [u8; 32]);

impl PublicKey {
    #[allow(dead_code)]
    pub fn try_from_hex_string(v: &str) -> Result<PublicKey, Error> {
        let vec = hex::decode(v)?;
        Ok(PublicKey(
            vec.try_into().map_err(|_| Error::WrongLengthHexString)?,
        ))
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(PublicKeyVisitor)
    }
}

struct PublicKeyVisitor;

impl<'de> Visitor<'de> for PublicKeyVisitor {
    type Value = PublicKey;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a hexadecimal string representing 32 bytes")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let vec =
            hex::decode(v).map_err(|e| serde::de::Error::custom(E::custom(format!("{}", e))))?;
        let pkey: [u8; 32] = vec.try_into().map_err(|e: Vec<u8>| {
            E::custom(format!(
                "public key is not 32 bytes long. was {} bytes long.",
                e.len()
            ))
        })?;
        Ok(PublicKey(pkey))
    }
}
