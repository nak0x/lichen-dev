-- Groups (classes / study circles), their memberships, and posts.
--
-- A group IS a `Group` actor (so it federates and has an outbox/followers for
-- free): its row keys off the same username. Memberships are the *local* access
-- model (who may post, who administers); remote participants arrive as followers
-- of the group actor via Join, handled by the inbox.

CREATE TABLE groups (
    username   TEXT PRIMARY KEY REFERENCES actors(username) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    summary    TEXT,
    subject    TEXT,                 -- the study subject: "le castor", "la rivière"...
    created_by UUID REFERENCES accounts(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE memberships (
    group_username TEXT NOT NULL REFERENCES groups(username) ON DELETE CASCADE,
    account_id     UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    role           TEXT NOT NULL,     -- admin | member
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (group_username, account_id)
);
CREATE INDEX memberships_account_idx ON memberships (account_id);

-- Posts are the queryable read-model over the activities we publish: the same
-- Create is also appended to the author's outbox (activities) for federation,
-- but feeds, filtering and group views read from here, where the structure
-- (author, group, tags, published) is first-class.
CREATE TABLE posts (
    id              UUID PRIMARY KEY,
    author_username TEXT NOT NULL REFERENCES actors(username) ON DELETE CASCADE,
    group_username  TEXT REFERENCES groups(username) ON DELETE CASCADE,  -- null = personal
    content         TEXT,
    tags            TEXT[] NOT NULL DEFAULT '{}',
    object_id       TEXT NOT NULL,     -- the Note IRI
    activity        JSONB NOT NULL,    -- canonical Create (raw view + re-delivery)
    published       TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX posts_group_published_idx ON posts (group_username, published DESC);
CREATE INDEX posts_author_published_idx ON posts (author_username, published DESC);
CREATE INDEX posts_tags_idx ON posts USING GIN (tags);
