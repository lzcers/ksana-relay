use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct Unixtime(pub i64);
