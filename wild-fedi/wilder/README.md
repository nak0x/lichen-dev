# wilder — a federated naturalist platform for outdoor schools

`wilder` is an ActivityPub server for the LICHEN network (see `../README.md` for
the fediverse primer). It began as a bare, queue-backed ActivityPub server and
grew into a small platform: authenticated accounts with roles, groups and posts,
a personal workspace, environmental measurements, and a school admin space —
federating with other instances over ActivityPub.

## Architecture

Layered, dependency-inverted, and the composition root (`src/main.rs`) is the
only place that knows the concrete implementations.

- **`domain/`** — entities and their invariants (accounts, groups, posts, notes,
  taxonomy, measurements, announcements), independent of storage or transport.
- **`repository/`** — persistence *traits*; `repository/postgres/` implements
  them, `repository/memory.rs` is a `#[cfg(test)]` test double.
- **`cache/`** — Redis-backed, rebuildable state: sessions, last-visit marks,
  rate limiting.
- **`media/`** — a `MediaStore` port with S3/Garage and filesystem backends.
- **`crypto/`** — RSA keys + HTTP Signatures (hand-rolled) and argon2 passwords.
- **`federation/`** — the sole owner of outbound server-to-server I/O, with the
  allowlist/SSRF guard.
- **`queue/`** — a durable Redis-Stream job queue drained by a worker pool.
- **`service/`** — the use-case layer; `handlers.rs` is a Strategy dispatcher for
  inbound activity verbs (Follow/Join/Leave/Create/…).
- **`web/`** — the HTTP edge: the ActivityPub contracts, a public `/media/:id`,
  and the client REST API under `/api/v1/*` guarded by session + role extractors.

### Data placement

Durable, relational data lives in **Postgres** (accounts, actors, groups, posts,
notes, taxonomy, media metadata, measurements, announcements, the activity log).
High-write / rebuildable state lives in **Redis** (sessions, per-user last-visit,
rate-limit counters, the job queue). Media **bytes** live in **Garage** (or a local
directory); their metadata stays in Postgres.

## Running

The whole stack runs from `../` with Docker:

```bash
cd ..            # wild-fedi/
make stack       # postgres + redis + garage + wilder + wilder-web
```

- Server: http://localhost:8080 · Client: http://localhost:3000
- Migrations run automatically on boot; a demo actor, a sample determination key,
  and (on an empty instance) nothing else are seeded idempotently.
- `make services` runs the backend only; `make ui` the frontend only.

### Local development

```bash
cp .env.example .env     # adjust as needed
cargo run                # needs Postgres + Redis reachable (see .env)
```

Set `MEDIA_BACKEND=fs` to avoid running Garage (bytes go under `MEDIA_FS_ROOT`).

## Configuration (env)

| Variable | Default | Purpose |
|---|---|---|
| `WILDER_SCHEME` / `WILDER_DOMAIN` / `WILDER_BIND` | `http` / `127.0.0.1:8080` / `127.0.0.1:8080` | Federated identity + listen address |
| `DATABASE_URL` | local Postgres | Durable store |
| `REDIS_URL` | local Redis | Volatile store + queue |
| `MEDIA_BACKEND` | `s3` | `s3` (Garage) or `fs` (local directory) |
| `MEDIA_FS_ROOT` | `./data/media` | Directory for the `fs` backend |
| `S3_ENDPOINT` / `S3_BUCKET` / `S3_ACCESS_KEY` / `S3_SECRET_KEY` / `S3_REGION` | Garage defaults | Object storage (region must match Garage's `s3_region`) |
| `WILDER_FEDERATION_ALLOWLIST` | empty (open) | Vetted peer domains; also the SSRF guard |
| `WILDER_WORKERS` / `WILDER_QUEUE_CAPACITY` | `4` / `1024` | Worker pool + stream cap |

## Notes & trade-offs

- **sqlx queries are runtime-checked** (not the `query!` macros), so builds are
  hermetic — no database is needed to compile, and Docker builds stay simple. The
  cost is that SQL errors surface at run time; each phase is verified by running.
- **Garage self-bootstraps** in the compose stack: `garage server --single-node
  --default-bucket` creates the cluster layout, the access key, and the media
  bucket (with owner rights) from the `GARAGE_DEFAULT_*` env vars — no init job
  or manual `garage layout`/`bucket create` is needed. Note Garage requires an
  access key ≥ 8 chars and a secret ≥ 16 chars. For a zero-dependency single box,
  `MEDIA_BACKEND=fs` runs no object store at all.
- **Closed federation** (`WILDER_FEDERATION_ALLOWLIST`) implements the
  children's-network posture (parent README §6): the instance talks only to
  vetted peers, and private features (notes, research, determination traversal)
  never leave it.

## Verification

```bash
cd ..            # wild-fedi/
./scripts/smoke.sh          # end-to-end REST smoke against a running instance
cargo test --manifest-path wilder/Cargo.toml   # unit tests (no DB needed)
```
