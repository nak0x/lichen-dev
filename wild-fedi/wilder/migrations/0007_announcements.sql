-- School-published announcements: events (field outings, dates, places) and
-- news (project write-ups). Each is federated as a Create wrapping an AS `Event`
-- or `Article`; this row is the queryable read-model, as for posts.
CREATE TABLE announcements (
    id              UUID PRIMARY KEY,
    kind            TEXT NOT NULL,            -- event | news
    author_username TEXT NOT NULL REFERENCES actors(username) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,
    start_time      TIMESTAMPTZ,              -- events: when it happens
    location        TEXT,                     -- events: where
    object_id       TEXT NOT NULL,
    activity        JSONB NOT NULL,
    published       TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX announcements_kind_published_idx ON announcements (kind, published DESC);
