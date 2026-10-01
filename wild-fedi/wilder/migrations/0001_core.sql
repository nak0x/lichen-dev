-- Core federation schema: the tables backing the repository traits that already
-- existed in the in-memory store (actors, followers, the activity log). Later
-- migrations add the application domain (accounts, groups, posts, ...) on top.

-- Local actors this instance hosts. Keyed by username, which is also what the
-- WebFinger / actor URLs are built from. The private key lives here for now; a
-- production deployment would isolate it in a secret manager (the repository
-- trait is the seam that lets that change without touching callers).
CREATE TABLE actors (
    username         TEXT PRIMARY KEY,
    kind             TEXT NOT NULL,            -- Person | Group | Organization | Service | Application
    display_name     TEXT,
    summary          TEXT,
    private_key_pem  TEXT NOT NULL,
    public_key_pem   TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Remote followers of a local actor. We cache each follower's resolved inbox so
-- delivery never has to refetch the actor document.
CREATE TABLE followers (
    username    TEXT NOT NULL REFERENCES actors(username) ON DELETE CASCADE,
    actor_id    TEXT NOT NULL,                 -- remote follower IRI
    inbox       TEXT NOT NULL,                 -- delivery target (personal or shared inbox)
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (username, actor_id)           -- idempotent: a re-sent Follow cannot duplicate
);

-- The activity log: everything a local actor published (outbox) or accepted at
-- its inbox. Stored as JSONB so the canonical wire form is preserved verbatim,
-- with `published` lifted out for chronological ordering and feed queries.
CREATE TABLE activities (
    id          BIGSERIAL PRIMARY KEY,         -- internal, monotonic insertion order
    username    TEXT NOT NULL REFERENCES actors(username) ON DELETE CASCADE,
    box         TEXT NOT NULL,                 -- 'outbox' | 'inbox'
    activity    JSONB NOT NULL,
    published   TIMESTAMPTZ,                   -- parsed from the activity when present
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX activities_username_box_id_idx ON activities (username, box, id DESC);
