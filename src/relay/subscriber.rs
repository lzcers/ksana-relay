use super::{EventFilter, SubscriberEvent};
use crate::nostr::{ClientMessage, Event, Filter, RelayMessage};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    net::TcpStream,
    sync::broadcast::Receiver as BroadcastReceiver,
    sync::mpsc::{self, Receiver, Sender},
};
use tokio_tungstenite::{
    self,
    tungstenite::{Message, Result},
    WebSocketStream,
};

/// 一个消息的订阅者
/// 发送、接收 Relay 的消息
pub struct Subscriber {
    subscriptions: HashMap<String, Vec<Filter>>,
    socket_addr: SocketAddr,
    // 用户断线重连直接从 event_cache 取？
    // event_cache: Vec<Event>,
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
            subscriptions: HashMap::new(),
            socket_addr,
            sender,
            broadcast_receiver,
            writer,
            reader,
        }
    }
    pub fn start(mut self) {
        tokio::spawn(async move {
            info!("New WebSocket connection: {}", &self.socket_addr);
            loop {
                tokio::select! {
                    r = self.reader.next() => {
                        if let Some(Ok(msg)) = r {
                            self.on_client_message(msg).await.expect("on client message error")
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
                let msg_str =
                    serde_json::to_string(&msg).expect("serde json faild from relay message");
                if let Err(e) = self.writer.send(Message::Text(msg_str)).await {
                    error!("send message failed from brodcast: {}", e);
                }
            }
        }
        Ok(())
    }

    pub async fn on_client_message(&mut self, msg: Message) -> Result<()> {
        if let Message::Text(client_msg) = msg {
            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&client_msg) {
                match client_msg {
                    ClientMessage::Event(e) => {
                        // 持久化
                        if let Err(e) = self.sender.send(SubscriberEvent::Event(e)).await {
                            // 重发？
                            error!("Subscriber send msg to relay faild : {}", e);
                        };
                    }
                    // 订阅某个内容
                    // 需要向 Relay 一次性请求数据
                    ClientMessage::REQ(id, filters) => {
                        self.subscriptions.insert(id.clone(), filters.clone());
                        let (tx, rx) = mpsc::channel::<RelayMessage>(32);
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
        }
        Ok(())
    }

    pub async fn on_relay_message(&mut self, mut rx: Receiver<RelayMessage>) -> Result<()> {
        while let Some(msg) = rx.recv().await {
            let msg_str = serde_json::to_string(&msg).expect("serde json faild from relay message");
            if let Err(e) = self.writer.send(Message::Text(msg_str)).await {
                error!("send message failed from relay: {}", e);
            }
        }
        Ok(())
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
}
