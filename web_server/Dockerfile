# syntax=docker/dockerfile:1.7
#
# Multi-stage build for the guitar-io web_server.
#
#   Stage 1  frontend-builder   node:22-alpine  →  vite/tsc build of frontend/
#   Stage 2  rust-builder       rust:1-bookworm →  cargo release build w/ embed
#   Stage 3  runtime            distroless/cc   →  single static-ish binary
#
# Requires BuildKit (default in modern Docker). Build from repo root:
#   docker build -f web_server/Dockerfile -t guitar-io/score-server:latest .

# ─── Stage 1: build the Vite/TypeScript frontend ─────────────────────────────
FROM node:22-alpine AS frontend-builder

RUN corepack enable
WORKDIR /build/frontend

# Deps layer — invalidated only when the lockfile changes.
COPY web_server/frontend/package.json \
     web_server/frontend/pnpm-lock.yaml \
     web_server/frontend/pnpm-workspace.yaml \
     ./
RUN --mount=type=cache,id=pnpm,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile

COPY web_server/frontend/ ./
RUN pnpm build


# ─── Stage 2: build the Rust binary with the frontend embedded ───────────────
FROM rust:1-bookworm AS rust-builder

WORKDIR /build

# Copy the full workspace. Cargo caching is handled by BuildKit cache mounts
# below (target/ + ~/.cargo/registry), so we skip the stub-build trick and
# accept a full copy up front.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY guitarpro/ ./guitarpro/
COPY cli/ ./cli/
COPY web_server/ ./web_server/

# Replace any host-built frontend dist with the one produced in stage 1 so
# rust-embed picks up the container-built assets.
RUN rm -rf ./web_server/frontend/dist
COPY --from=frontend-builder /build/frontend/dist/ ./web_server/frontend/dist/

# target/ lives in a cache mount, so the binary must be copied out before the
# layer is torn down.
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=cargo-target,target=/build/target,sharing=locked \
    cargo build --release -p web_server --bin score_server --features embed \
 && cp /build/target/release/score_server /score_server \
 && strip /score_server

# Empty scores dir to COPY into the distroless stage (no shell to mkdir there).
RUN mkdir -p /empty/data/scores


# ─── Stage 3: minimal runtime (distroless, glibc) ────────────────────────────
FROM gcr.io/distroless/cc-debian12

COPY --from=rust-builder /score_server /usr/local/bin/score_server
COPY --from=rust-builder --chown=nonroot:nonroot /empty/data /data

USER nonroot:nonroot
WORKDIR /data
EXPOSE 3000
ENV RUST_LOG=info

# Bind to 0.0.0.0 so the container is reachable via the published port and
# suppress the browser-auto-open (there is no display in a container).
ENTRYPOINT ["/usr/local/bin/score_server", \
            "--no-open", \
            "--host", "0.0.0.0", \
            "--port", "3000", \
            "--root", "/data/scores"]
