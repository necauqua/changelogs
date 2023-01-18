FROM rust:1.66-slim AS builder

# RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build

# cache the crates.io index thing
RUN cargo search --limit 0

# make a dumb program to cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release --locked && rm -r target/release

COPY src src

RUN cargo build --release --locked
# --target x86_64-unknown-linux-musl

# something.. something.. tls issues with musl, so we cant use it and thus cant use scratch/alpine
# # FROM scratch
# FROM alpine/git
# COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/changelogs /bin/changelogs
# ENTRYPOINT ["/bin/changelogs"]

FROM debian:stable-slim

RUN apt update -y
RUN apt install -y git

COPY --from=builder /build/target/release/changelogs /bin/changelogs

ENTRYPOINT ["/bin/changelogs"]
