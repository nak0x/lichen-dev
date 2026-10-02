-- Private personal documents: self-notes and research write-ups. Scoped to one
-- owner and never federated — the instance keeps them to itself (README §6).
-- `kind` distinguishes a short note from a longer research document.
CREATE TABLE notes (
    id             UUID PRIMARY KEY,
    owner_username TEXT NOT NULL REFERENCES actors(username) ON DELETE CASCADE,
    kind           TEXT NOT NULL,          -- note | research
    title          TEXT,
    content        TEXT NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX notes_owner_kind_idx ON notes (owner_username, kind, updated_at DESC);

-- The determination key: a taxonomy tree. Each non-leaf node poses a
-- discriminating `question`; its children are the possible answers. Walking from
-- a root to a `is_species` leaf identifies a living thing from general to precise.
-- Curated, instance-wide reference data (seeded on first boot).
CREATE TABLE taxon_nodes (
    id         UUID PRIMARY KEY,
    parent_id  UUID REFERENCES taxon_nodes(id) ON DELETE CASCADE,  -- null = a root
    rank       TEXT,                        -- "Règne", "Classe"... (free text)
    name       TEXT NOT NULL,
    vernacular TEXT,                         -- common name, for species leaves
    is_species BOOLEAN NOT NULL DEFAULT false,
    question   TEXT,                         -- posed to choose among this node's children
    position   INT NOT NULL DEFAULT 0,       -- ordering among siblings
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX taxon_parent_idx ON taxon_nodes (parent_id, position);
