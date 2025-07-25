FROM rust:1.88 AS build
WORKDIR /usr/src

# Install build dependencies
RUN USER=root rustup target add wasm32-unknown-unknown
RUN USER=root cargo install --locked wasm-bindgen-cli

WORKDIR /usr/src/nchoputa
COPY ./ ./
RUN target=release make

# Copy the statically-linked binary into a final minimal container
# FROM scratch
FROM debian:buster-slim

WORKDIR /opt
COPY --from=build /usr/src/nchoputa/target/release/nchoputa .
COPY static ./static/
COPY --from=build /usr/src/nchoputa/static/viewer* ./static/

USER 1000
EXPOSE 8999
CMD ["./nchoputa", "--port", "8999"]
