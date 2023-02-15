use super::{event::Event, EventKind, Id, PublicKey, Unixtime};

use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Filter {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ids: Vec<Id>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub authors: Vec<PublicKey>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub kinds: Vec<EventKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "#e")]
    #[serde(default)]
    pub e: Vec<Id>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "#p")]
    #[serde(default)]
    pub p: Vec<PublicKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub since: Option<Unixtime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub until: Option<Unixtime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum ClientMessage {
    /// An event
    Event(Event),
    REQ(String, Vec<Filter>),
    Close(String),
}

impl<'de> Deserialize<'de> for ClientMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ClientMessageVisitor)
    }
}
struct ClientMessageVisitor;
impl<'de> Visitor<'de> for ClientMessageVisitor {
    type Value = ClientMessage;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, " a sequence of strings")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let tagname: &str = match seq.next_element()? {
            Some(e) => e,
            None => panic!("unknown client message"),
        };

        match tagname {
            "REQ" => {
                let oid = seq.next_element::<String>()?;
                let mut filters: Vec<Filter> = vec![];

                if let Some(id) = oid {
                    loop {
                        if let Some(f) = seq.next_element()? {
                            filters.push(f);
                        } else {
                            break;
                        }
                    }
                    Ok(ClientMessage::REQ(id, filters))
                } else {
                    panic!("unknown REQ msg")
                }
            }
            "EVENT" => {
                let oe = seq.next_element::<Event>()?;
                if let Some(e) = oe {
                    Ok(ClientMessage::Event(e))
                } else {
                    panic!("unknown EVENT msg")
                }
            }
            "CLOSE" => {
                let oid = seq.next_element::<String>()?;
                if let Some(id) = oid {
                    Ok(ClientMessage::Close(id))
                } else {
                    panic!("unknown CLOSE msg")
                }
            }
            _ => panic!("unknown client message"),
        }
    }
}

impl Serialize for ClientMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        match self {
            ClientMessage::Event(e) => {
                seq.serialize_element("EVENT")?;
                seq.serialize_element(e)?;
            }
            ClientMessage::REQ(id, filter) => {
                seq.serialize_element("REQ")?;
                seq.serialize_element(id)?;
                seq.serialize_element(filter)?;
            }
            ClientMessage::Close(id) => {
                seq.serialize_element("CLOSE")?;
                seq.serialize_element(id)?;
            }
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use super::Filter;
    use crate::nostr::{EventKind, Id, PublicKey};

    #[test]
    fn test_serde_filter() {
        let filter = Filter {
            ids: vec![Id::try_from_hex_string(
                "5cd7d34f0ad72dac07cae33c4ed784a835f766343a3e7e74f9c7d6b8e9cca449",
            )
            .unwrap()],
            authors: vec![PublicKey::try_from_hex_string(
                "9ab2f6b34894c95e7e36cea26fecf8dea88f383ed8d6e652b1a8d749695825e4",
            )
            .unwrap()],
            kinds: vec![EventKind::Metadata],
            e: vec![Id::try_from_hex_string(
                "5cd7d34f0ad72dac07cae33c4ed784a835f766343a3e7e74f9c7d6b8e9cca449",
            )
            .unwrap()],
            p: vec![PublicKey::try_from_hex_string(
                "9ab2f6b34894c95e7e36cea26fecf8dea88f383ed8d6e652b1a8d749695825e4",
            )
            .unwrap()],
            since: None,
            until: None,
            limit: Some(200),
        };
        let str = serde_json::to_string(&filter);
        println!("{:?}", str);
    }
    #[test]
    fn test_serde_event() {
        Filter {
            ids: vec![Id::try_from_hex_string(
                "5cd7d34f0ad72dac07cae33c4ed784a835f766343a3e7e74f9c7d6b8e9cca449",
            )
            .unwrap()],
            authors: vec![PublicKey::try_from_hex_string(
                "9ab2f6b34894c95e7e36cea26fecf8dea88f383ed8d6e652b1a8d749695825e4",
            )
            .unwrap()],
            kinds: vec![EventKind::Metadata],
            e: vec![Id::try_from_hex_string(
                "5cd7d34f0ad72dac07cae33c4ed784a835f766343a3e7e74f9c7d6b8e9cca449",
            )
            .unwrap()],
            p: vec![PublicKey::try_from_hex_string(
                "9ab2f6b34894c95e7e36cea26fecf8dea88f383ed8d6e652b1a8d749695825e4",
            )
            .unwrap()],
            since: None,
            until: None,
            limit: Some(200),
        };
    }
}
