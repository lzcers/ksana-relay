mod error;

use crate::nostr::{self, Event, EventKind, Id, PublicKey, Signature, Tag, Unixtime};
pub use error::Error;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

// pub struct DBEvent(Event);

impl Database {
    pub async fn connect(url: &str) -> Result<Self, Error> {
        let pool = SqlitePool::connect(url).await?;
        Ok(Database { pool })
    }

    pub async fn save_event(&self, e: &Event) -> Result<(), Error> {
        let mut conn = self.pool.acquire().await?;
        let id = e.id.0.as_slice();
        let pubkey = e.pubkey.0.as_slice();
        let created_at = e.created_at.0;
        let kind: u64 = e.kind.into();
        let kind_u32 = kind as u32;
        let tags = serde_json::ser::to_string(&e.tags)
            .expect("save event faild, serialization tags faild!");
        let content = &e.content;
        let sig_bytes = e.sig.0.to_bytes();
        let sig = sig_bytes.as_slice();
        sqlx::query!(
            r#"
            INSERT INTO nostr_events (id, pubkey, created_at, kind, tags, content, sig)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            id,
            pubkey,
            created_at,
            kind_u32,
            tags,
            content,
            sig
        )
        .execute(&mut conn)
        .await?
        .last_insert_rowid();
        Ok(())
    }

    pub async fn get_events(&self) -> Result<Vec<Event>, Error> {
        let mut coon = self.pool.acquire().await?;
        let events = sqlx::query!("SELECT * FROM nostr_events")
            .map(|row| {
                if let Some(id) = row.id {
                    let tags = serde_json::from_str::<Vec<Tag>>(&row.tags)
                        .expect("serde tags faild from database row!");
                    Event {
                        id: Id(id.try_into().expect("serde id faild from database row!")),
                        pubkey: PublicKey(
                            row.pubkey
                                .try_into()
                                .expect("serde pubkey faild from database row!"),
                        ),
                        created_at: Unixtime(row.created_at),
                        kind: EventKind::from(row.kind as u64),
                        tags,
                        content: row.content,
                        sig: Signature::try_from_vec_u8(row.sig)
                            .expect("serde sig faild from database row!"),
                    }
                } else {
                    panic!("serde id faild from database row!");
                }
            })
            .fetch_all(&mut coon)
            .await?;
        Ok(events)
    }

    pub async fn delete_event(
        &mut self,
        id: &nostr::Id,
        pubkey: &nostr::PublicKey,
    ) -> Result<u64, Error> {
        let mut conn = self.pool.acquire().await?;
        let id_slice = id.0.as_slice();
        let pubkey_slice = pubkey.0.as_slice();
        let r = sqlx::query!(
            "DELETE FROM nostr_events WHERE id = ? AND pubkey = ?",
            id_slice,
            pubkey_slice
        )
        .execute(&mut conn)
        .await?
        .rows_affected();
        Ok(r)
    }

    #[allow(dead_code)]
    pub async fn get_event_by_id(&self, id: &nostr::Id) -> Result<(), Error> {
        let mut coon = self.pool.acquire().await?;
        let id_slice = id.0.as_slice();
        let _events = sqlx::query!("SELECT * FROM nostr_events WHERE id = ?", id_slice)
            .fetch_one(&mut coon)
            .await;
        todo!()
    }
}
