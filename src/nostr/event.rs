use super::{Error, EventKind, Id, PrivateKey, PublicKey, Signature, Tag, Unixtime};
use k256::schnorr::signature::DigestVerifier;
use k256::schnorr::VerifyingKey;
use k256::sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub struct PreEvent {
    /// The public key of the actor who is creating the event
    pub pubkey: PublicKey,
    /// The time at which the event was created
    pub created_at: Unixtime,
    /// The kind of event
    pub kind: EventKind,
    /// A set of tags that apply to the event
    pub tags: Vec<Tag>,
    /// The content of the event
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Event {
    // 32-bytes lowercase hex-encoded sha256 of the the serialized event data
    pub id: Id,

    // <32-bytes lowercase hex-encoded public key of the event creator>,
    pub pubkey: PublicKey,

    // <unix timestamp in seconds>
    pub created_at: Unixtime,

    pub kind: EventKind,

    pub tags: Vec<Tag>,

    pub content: String,

    // "sig": <64-bytes signature of the sha256 hash of the serialized event data, which is the same as the "id" field>
    pub sig: Signature,
}

macro_rules! serialize_inner_event {
    ($pubkey: expr, $created_at:expr, $kind:expr, $tags: expr, $content: expr) => {{
        format!(
            "[0,{},{},{},{},{}]",
            serde_json::to_string($pubkey)?,
            serde_json::to_string($created_at)?,
            serde_json::to_string($kind)?,
            serde_json::to_string($tags)?,
            serde_json::to_string($content)?,
        )
    }};
}

impl Event {
    #[allow(dead_code)]
    pub fn new(input: PreEvent, privkey: &PrivateKey) -> Result<Event, Error> {
        let id = Self::hash(&input)?;
        let sig = privkey.sign_id(id)?;
        Ok(Event {
            id,
            pubkey: input.pubkey,
            created_at: input.created_at,
            kind: input.kind,
            tags: input.tags,
            content: input.content,
            sig,
        })
    }

    pub fn verify(&self) -> Result<(), Error> {
        let serialized = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        //  检验签名
        let digest = Sha256::new_with_prefix(&serialized);
        let verifier = VerifyingKey::from_bytes(&self.pubkey.0)?;
        verifier.verify_digest(digest, &self.sig)?;

        // 上述签名校验保证了消息内容和签名是相符的，但是换个 Id 但不改变内容和签名的情况下上述校验依旧能够
        // 所以还要校验 ID
        // 校验 id 是否一致
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let id = hasher.finalize();
        if *id != self.id.0 {
            Err(Error::HashMismatch)
        } else {
            Ok(())
        }
    }

    // hash 计算出 id
    pub fn hash(input: &PreEvent) -> Result<Id, Error> {
        let serialized: String = serialize_inner_event!(
            &input.pubkey,
            &input.created_at,
            &input.kind,
            &input.tags,
            &input.content
        );
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let id = hasher.finalize();
        let id: [u8; 32] = id.into();
        Ok(Id(id))
    }
}
