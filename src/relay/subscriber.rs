use super::{EventFilter, SubscriberEvent};
use crate::nostr::{ClientMessage, Event, EventKind, Filter, PublicKey, RelayMessage, Tag};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info};
use std::{
    collections::HashMap,
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    net::TcpStream,
    sync::broadcast::Receiver as BroadcastReceiver,
    sync::mpsc::Sender,
    sync::{oneshot, oneshot::Receiver as OneshotReciver},
};
use tokio_tungstenite::{
    self,
    tungstenite::{Message, Result},
    WebSocketStream,
};

struct UserInfo {
    #[allow(dead_code)]
    pubkey: PublicKey,
}

/// 一个消息的订阅者
/// 发送、接收 Relay 的消息
pub struct Subscriber {
    user_info: Option<UserInfo>,
    subscriptions: HashMap<String, Vec<Filter>>,
    socket_addr: SocketAddr,
    writer: SplitSink<WebSocketStream<TcpStream>, Message>,
    reader: SplitStream<WebSocketStream<TcpStream>>,

    sender: Sender<SubscriberEvent>,
    broadcast_receiver: BroadcastReceiver<Event>,
}

impl Subscriber {
    pub fn new(
        socket_addr: SocketAddr,
        socket_stream: WebSocketStream<TcpStream>,
        sender: Sender<SubscriberEvent>,
        broadcast_receiver: BroadcastReceiver<Event>,
    ) -> Self {
        let (writer, reader) = socket_stream.split();
        Subscriber {
            user_info: None,
            subscriptions: HashMap::new(),
            socket_addr,
            sender,
            broadcast_receiver,
            writer,
            reader,
        }
    }
    //todo: 考虑为 Subscriber 加入状态和身份，控制订阅权限
    pub fn start(mut self) {
        tokio::spawn(async move {
            info!("New WebSocket connection: {}", &self.socket_addr);
            if self.user_info.is_none() {
                self.send_auth_event().await;
            }
            loop {
                tokio::select! {
                    r = self.reader.next() => {
                        if let Some(Ok(msg)) = r {
                            self.on_client_message(msg).await.expect("on client message error");
                        } else {
                            info!("Client disconnected: {}", &self.socket_addr);
                            break;
                        }
                    },
                    Ok(evt) = self.broadcast_receiver.recv() => {
                        self.on_brodcast_message(&evt).await.expect("on brodcast message error");
                    }
                }
            }
        });
    }

    pub async fn on_brodcast_message(&mut self, evt: &Event) -> Result<()> {
        if let Some(msgs) = self.is_subscribed(evt) {
            for msg in msgs {
                self.send_relay_message(&msg).await;
            }
        }
        Ok(())
    }

    pub async fn on_client_message(&mut self, msg: Message) -> Result<()> {
        if let Message::Text(client_msg) = msg {
            match serde_json::from_str::<ClientMessage>(&client_msg) {
                Ok(client_msg) => {
                    match client_msg {
                        ClientMessage::Auth(e) => {
                            if self.check_auth_event(&e) {
                                let key_str = e.pubkey.as_hex_string();
                                self.user_info = Some(UserInfo { pubkey: e.pubkey });
                                let auth_info = RelayMessage::Notice(
                                    format!("Authentication success with pubkey: {}", key_str)
                                        .to_string(),
                                );
                                self.send_relay_message(&auth_info).await;
                            }
                        }
                        ClientMessage::Event(e) => {
                            if self.user_info.is_none() {
                                self.send_auth_event().await;
                                return Ok(());
                            }
                            if let Err(err) = e.verify() {
                                error!("msg verify failed！{:?}, event: {:?}", err, e);
                                return Ok(());
                            }
                            // 持久化
                            if let Err(e) = self.sender.send(SubscriberEvent::Event(e)).await {
                                // 重发？
                                error!("Subscriber send msg to relay faild : {}", e);
                            };
                        }
                        // 订阅某个内容
                        // 需要向 Relay 一次性请求数据
                        ClientMessage::REQ(id, filters) => {
                            if self.user_info.is_none() {
                                self.send_auth_event().await;
                                return Ok(());
                            }
                            self.subscriptions.insert(id.clone(), filters.clone());
                            let (tx, rx) = oneshot::channel();
                            match self
                                .sender
                                .send(SubscriberEvent::Req(id, filters, tx))
                                .await
                            {
                                Ok(_) => {
                                    self.on_relay_message(rx)
                                        .await
                                        .expect("on relay message faild!");
                                }
                                Err(e) => {
                                    error!("send msg to relay faild: {}", e);
                                }
                            }
                        }
                        // 取消订阅
                        ClientMessage::Close(id) => {
                            self.subscriptions.remove(&id);
                        }
                    }
                }
                Err(e) => error!("wrong client message format: {}", e),
            }
        }
        Ok(())
    }

    pub async fn on_relay_message(&mut self, rx: OneshotReciver<Vec<RelayMessage>>) -> Result<()> {
        if let Ok(msgs) = rx.await {
            for rmsg in msgs {
                self.send_relay_message(&rmsg).await;
            }
        }
        Ok(())
    }

    pub fn check_auth_event(&self, e: &Event) -> bool {
        // To verify AUTH messages, relays must ensure:
        // that the kind is 22242;
        // that the event created_at is close (e.g. within ~10 minutes) of the current time;
        // that the "challenge" tag matches the challenge sent before;
        // that the "relay" tag matches the relay URL:
        // URL normalization techniques can be applied. For most cases just checking if the domain name is correct should be enough.
        if let Err(err) = e.verify() {
            error!("auth msg verify failed！{:?}, event: {:?}", err, e);
            return false;
        }
        if e.kind != EventKind::Auth {
            return false;
        }

        let (mut relay, mut challenge) = ("", "");
        for tag in &e.tags {
            match tag {
                Tag::Relay(r) => {
                    relay = r;
                }
                Tag::Challenge(c) => {
                    challenge = c;
                }
                _ => {}
            }
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("get now time faild!")
            .as_secs();
        let duration = now - (e.created_at.0) as u64;
        let ten_minutes = 10 * 60 as u64;

        if ten_minutes - duration > 0 && challenge == "ksana.io" && relay == "wss://relay.ksana.net"
        {
            true
        } else {
            false
        }
    }
    pub async fn send_auth_event(&mut self) {
        if self.user_info.is_none() {
            let notice = &RelayMessage::Notice("restricted: we can't serve DMs to unauthenticated users, does your client implement NIP-42?".to_string());
            self.send_relay_message(notice).await;
            self.send_relay_message(&RelayMessage::Auth("ksana.io".to_string()))
                .await;
        }
    }
    pub fn is_subscribed(&self, event: &Event) -> Option<Vec<RelayMessage>> {
        let mut rmsgs: Vec<RelayMessage> = vec![];
        for (id, filters) in &self.subscriptions {
            if EventFilter::any_filter(&event, filters) {
                rmsgs.push(RelayMessage::Event(id.to_owned(), event.clone()));
            }
        }
        if rmsgs.is_empty() {
            None
        } else {
            Some(rmsgs)
        }
    }
    pub async fn send_relay_message(&mut self, relay_message: &RelayMessage) {
        let msg_str = serde_json::to_string(&relay_message).expect("msg serde faild!");
        if let Err(e) = self.writer.send(Message::Text(msg_str)).await {
            if let tokio_tungstenite::tungstenite::Error::ConnectionClosed = e {
                error!("send relay msg error: {}", e)
            }
        }
    }
}
