-- Media metadata. The bytes live in the object store (MinIO); this row is the
-- durable index that maps a public id to its object key, owner and content type.
CREATE TABLE media (
    id           UUID PRIMARY KEY,
    owner_username TEXT NOT NULL REFERENCES actors(username) ON DELETE CASCADE,
    object_key   TEXT NOT NULL,            -- key within the bucket
    content_type TEXT NOT NULL,
    byte_size    BIGINT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX media_owner_idx ON media (owner_username);

-- Actors gain an avatar (ActivityPub `icon`), served as a media URL.
ALTER TABLE actors ADD COLUMN icon_url TEXT;
