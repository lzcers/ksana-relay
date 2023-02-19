use super::SubscriberEvent;
use crate::{
    database,
    nostr::{Event, RelayMessage},
    relay::EventFilter,
};
use log::{error, info};
use tokio::sync::{broadcast::Sender, mpsc::Receiver};

pub struct Relay {
    db: database::Database,
    subscriber_msg_receiver: Receiver<SubscriberEvent>,
    broadcast_sender: Sender<Event>,
    events: Vec<Event>,
}

impl Relay {
    pub fn new(
        db: database::Database,
        rec: Receiver<SubscriberEvent>,
        broadcast_sender: Sender<Event>,
    ) -> Relay {
        Relay {
            db,
            subscriber_msg_receiver: rec,
            broadcast_sender,
            events: vec![],
        }
    }

    pub fn start(mut self) {
        info!("relay start!");
        tokio::spawn(async move {
            info!("init relay by get events from db!");
            match self.db.get_events().await {
                Ok(events) => self.events = events,
                Err(e) => {
                    error!("get events faild from db: {}", e)
                }
            }
            self.on_subscriber_event().await;
        });
    }

    pub async fn on_subscriber_event(&mut self) {
        info!("on subscriber event...");

        while let Some(v) = self.subscriber_msg_receiver.recv().await {
            match v {
                SubscriberEvent::Event(e) => {
                    self.persist_event(&e).await;
                    self.events.push(e.clone());
                    self.broadcast_sender
                        .send(e)
                        .expect("broadcast event faild by relay");
                }
                SubscriberEvent::Req(id, filters, sx) => {
                    let mut events: Vec<Event> = vec![];
                    for event in &self.events {
                        for filter in &filters {
                            if EventFilter::filter(event, filter) {
                                events.push(event.clone())
                            }
                        }
                    }
                    for e in events {
                        if let Err(send_err) = sx.send(RelayMessage::Event(id.clone(), e)).await {
                            error!("relay msg send error: {}", send_err);
                        }
                    }
                }
            }
        }
        info!("on_subscriber_event end");
    }

    /// 将收到的 event 持久化到数据库中
    pub async fn persist_event(&mut self, e: &Event) {
        if let Err(e) = self.db.save_event(&e).await {
            error!("new event save faild: {}", e);
        }
    }
}
