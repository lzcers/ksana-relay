use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

use super::event::Event;

#[derive(Debug, Clone)]
pub enum RelayMessage {
    Auth(String),
    Event(String, Event),
    Notice(String),
}

impl Serialize for RelayMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;

        match self {
            RelayMessage::Auth(str) => {
                seq.serialize_element("AUTH")?;
                seq.serialize_element(str)?;
            }
            RelayMessage::Event(id, e) => {
                seq.serialize_element("EVENT")?;
                seq.serialize_element(id)?;
                seq.serialize_element(e)?;
            }
            RelayMessage::Notice(str) => {
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(str)?;
            }
        }
        seq.end()
    }
}

struct RelayMessageVisitor;

impl<'de> Deserialize<'de> for RelayMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessage;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RelayMessage is array")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let tagname: &str = match seq.next_element()? {
            Some(v) => v,
            None => panic!("unknown tagname in RelayMessage"),
        };

        match tagname {
            "EVENT" => {
                if let (Some(id), Some(event)) = (seq.next_element()?, seq.next_element()?) {
                    return Ok(RelayMessage::Event(id, event));
                } else {
                    panic!("id or envet not found in RelayMessage::Event");
                }
            }
            "AUTH" => {
                if let Some(str) = seq.next_element()? {
                    return Ok(RelayMessage::Auth(str));
                } else {
                    panic!("invalid auth msg in RelayMessage::Auth");
                }
            }
            "NOTICE" => {
                if let Some(str) = seq.next_element()? {
                    return Ok(RelayMessage::Notice(str));
                } else {
                    panic!("content not found in RelayMessage::Notice");
                }
            }
            _ => panic!("unknown RelayMessage"),
        }
    }
}
