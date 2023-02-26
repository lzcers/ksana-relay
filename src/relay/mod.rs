mod filter;
mod relayer;
mod subscriber;

pub use filter::*;
pub use relayer::*;
pub use subscriber::*;
use tokio::sync::oneshot::Sender;

use crate::nostr::{Event, Filter, RelayMessage};

pub enum SubscriberEvent {
    Event(Event),
    Req(String, Vec<Filter>, Sender<Vec<RelayMessage>>),
}
