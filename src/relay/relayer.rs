use super::SubscriberEvent;
use crate::{
    database,
    nostr::{Event, EventKind, Id, PublicKey, RelayMessage, Tag},
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
                    self.process_event(e).await;
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

    pub async fn process_event(&mut self, evt: Event) {
        match evt.kind {
            EventKind::TextNote => {
                self.persist_event(&evt).await;
                self.events.push(evt.clone());
            }
            EventKind::EventDeletion => {
                let mut ids: Vec<&Id> = vec![];
                evt.tags.iter().for_each(|tag| {
                    if let Tag::Event { id, .. } = tag {
                        ids.push(id);
                    }
                });
                for id in ids {
                    if self.delete_event(id, &evt.pubkey).await > 0 {
                        self.events = self
                            .events
                            .iter()
                            .filter(|e| e.id != *id)
                            .map(|e| e.clone())
                            .collect::<Vec<Event>>();
                    }
                }
            }
            EventKind::EncryptedDirectMessage => {
                self.persist_event(&evt).await;
                self.events.push(evt.clone());
            }
            _ => {}
        }
        self.broadcast_sender
            .send(evt)
            .expect("broadcast event faild by relay");
    }

    pub async fn delete_event(&mut self, id: &Id, pubkey: &PublicKey) -> u64 {
        let result = self.db.delete_event(id, pubkey).await;
        match result {
            Ok(row) => row,
            Err(e) => {
                error!("delete event faild: {}", e);
                0 as u64
            }
        }
    }

    /// 将收到的 event 持久化到数据库中
    pub async fn persist_event(&mut self, e: &Event) {
        if let Err(e) = self.db.save_event(&e).await {
            error!("new event save faild: {}", e);
        }
    }
}
