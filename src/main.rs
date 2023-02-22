mod database;
mod error;
mod nostr;
mod relay;
use dotenv::dotenv;
use error::RelayError;
use log::*;
use nostr::Event;
use relay::{Relay, Subscriber, SubscriberEvent};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc},
};
use tokio_tungstenite::{self, accept_async, tungstenite::Result};

#[tokio::main]
async fn main() -> Result<(), RelayError> {
    dotenv().ok();
    env_logger::init();

    let db = database::Database::connect(
        &dotenv::var("DATABASE_URL").expect("can't found DATABASE_URL in env."),
    )
    .await?;
    let (subscriber_msg_sender, subscriber_msg_receiver) = mpsc::channel::<SubscriberEvent>(32);
    let (broadcast_sender, broadcast_receiver) = broadcast::channel::<Event>(32);

    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);
    Relay::new(db, subscriber_msg_receiver, broadcast_sender).start();

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        let ws_stream = accept_async(stream).await.expect("Failed to accept");

        Subscriber::new(
            peer,
            ws_stream,
            subscriber_msg_sender.clone(),
            broadcast_receiver.resubscribe(),
        )
        .start();
    }
    Ok(())
}
