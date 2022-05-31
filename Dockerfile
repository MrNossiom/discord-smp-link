# Build step
FROM rust:latest as builder
ENV RUSTFLAGS="-C target-cpu=native"

# Install deps libs for dyn linking
RUN apt update && apt install -y libpq-dev && apt clean

# Create a new empty shell project
RUN USER=root cargo new --bin discord_smp_link
WORKDIR /discord_smp_link

# Copy over your manifests
COPY Cargo.toml Cargo.lock ./

# Build and cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy your source tree
COPY ./src ./src
COPY ./server ./server
COPY ./askama.toml ./

# Build for release
RUN rm ./target/release/deps/discord_smp_link*
RUN cargo build --release
RUN cargo install --path .

# Run step
FROM debian:buster-slim

RUN apt update -y && apt install -y libpq5 && rm -rf /var/lib/apt/lists/*

# Create a folder to recover logs and get .env file
WORKDIR /tmp/discord_smp_link/

# Copy the build artifact from the build stage
COPY --from=builder /usr/local/cargo/bin/discord_smp_link ./discord_smp_link

# Set the startup command to run your binary
CMD ["./discord_smp_link"]
