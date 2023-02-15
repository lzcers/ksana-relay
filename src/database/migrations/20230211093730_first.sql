-- Add migration script here
CREATE TABLE IF NOT EXISTS nostr_events (
    id BLOB PRIMARY KEY,
    pubkey BLOB NOT NULL,
    created_at INTEGER NOT NULL, -- Unix timestamp
    kind INTEGER NOT NULL,
    tags TEXT NOT NULL,
    content TEXT NOT NULL,
    sig BLOB NOT NULL
)