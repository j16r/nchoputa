FROM rust:1.65 AS build
WORKDIR /usr/src

# Install build dependencies
RUN USER=root rustup target add wasm32-unknown-unknown
RUN USER=root cargo install wasm-bindgen-cli

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
RUN USER=root cargo new --vcs none nchoputa
WORKDIR /usr/src/nchoputa
RUN USER=root cargo new --vcs none --lib shared
COPY Cargo.toml Cargo.lock ./
COPY shared/Cargo.toml shared/Cargo.lock shared/
RUN cargo build --release

# Viewer is not a workspace member, so it's handled separately
RUN USER=root cargo new --vcs none --lib viewer
WORKDIR /usr/src/nchoputa/viewer
COPY viewer/Cargo.toml viewer/Cargo.lock ./
COPY viewer/.cargo/ ./.cargo/
RUN cargo build --release

WORKDIR /usr/src/nchoputa

# Remove outputs from build above, they'll confuse make
RUN rm target/release/nchoputa viewer/target/wasm32-unknown-unknown/release/viewer.wasm

# Copy the source and build the application, ordered by churn least to most
COPY GNUmakefile ./
COPY data/ ./data/
COPY shared/src/ ./shared/src/
COPY src/ ./src/
COPY viewer/src/ ./viewer/src/
RUN target=release make install

# Copy the statically-linked binary into a final minimal container
# FROM scratch
FROM debian:buster-slim

WORKDIR /opt
COPY --from=build /usr/local/cargo/bin/nchoputa .
COPY static ./static

USER 1000
EXPOSE 8999
CMD ["./nchoputa", "--port", "8999"]
