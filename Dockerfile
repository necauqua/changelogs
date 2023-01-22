FROM rust:1.65-slim AS builder

RUN apt update -y
RUN apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build

# cache the crates.io index thing
RUN cargo search --limit 0

# make a dumb program to cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release --locked && rm -r target/release

COPY src src

RUN cargo build --release --locked --target x86_64-unknown-linux-musl

FROM alpine/git

RUN git config --global safe.directory '*'

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/changelogs /bin/changelogs

ENTRYPOINT ["/bin/changelogs"]
