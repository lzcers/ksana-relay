use std::fmt;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventKind {
    Metadata,
    TextNote,
}

impl From<EventKind> for u32 {
    fn from(value: EventKind) -> Self {
        use EventKind::*;
        match value {
            Metadata => 0,
            TextNote => 1,
        }
    }
}

impl From<EventKind> for u64 {
    fn from(value: EventKind) -> Self {
        use EventKind::*;
        match value {
            Metadata => 0,
            TextNote => 1,
        }
    }
}

impl From<i64> for EventKind {
    fn from(u: i64) -> Self {
        use EventKind::*;
        match u {
            0 => Metadata,
            1 => TextNote,
            _ => panic!(),
        }
    }
}
impl From<u64> for EventKind {
    fn from(u: u64) -> Self {
        use EventKind::*;
        match u {
            0 => Metadata,
            1 => TextNote,
            _ => panic!(),
        }
    }
}

impl Serialize for EventKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let u = From::from(*self);
        serializer.serialize_u64(u)
    }
}

impl<'de> Deserialize<'de> for EventKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(EventKindVisitor)
    }
}

struct EventKindVisitor;

impl Visitor<'_> for EventKindVisitor {
    type Value = EventKind;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "an unsigned number that matches a known EventKind")
    }

    fn visit_u64<E>(self, v: u64) -> Result<EventKind, E>
    where
        E: serde::de::Error,
    {
        Ok(From::<u64>::from(v))
    }
}
