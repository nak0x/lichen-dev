# syntax=docker/dockerfile:1
# --- Étage de compilation ---------------------------------------------------
FROM rust:1-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
# On met en cache le registre cargo et le dossier `target` entre les builds, puis
# on copie le binaire HORS du montage cache : un montage cache ne persiste pas
# dans la couche image, contrairement à /usr/local/bin.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && cp target/release/wilder /usr/local/bin/wilder

# --- Image d'exécution ------------------------------------------------------
FROM debian:bookworm-slim AS runtime
# ca-certificates pour d'éventuels appels https vers des instances distantes ;
# curl seulement pour le healthcheck de docker-compose.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/wilder /usr/local/bin/wilder
# Dans un conteneur, il faut écouter sur toutes les interfaces (pas 127.0.0.1).
ENV WILDER_BIND=0.0.0.0:8080
EXPOSE 8080
CMD ["wilder"]
