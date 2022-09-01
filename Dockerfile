# Build step
FROM rust:latest as builder
ENV RUSTFLAGS="-C target-cpu=native"

# Install developement libraries and headers
RUN apt update && apt install -y default-libmysqlclient-dev && apt clean

# Create a new empty shell project
RUN USER=root cargo new --bin discord_smp_link
WORKDIR /discord_smp_link/

# Copy over your manifests
COPY Cargo.toml Cargo.lock ./

# Build and cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy your source tree
COPY ./src ./src
COPY ./templates ./templates
COPY ./migrations ./migrations

# Build for release
RUN rm ./target/release/deps/discord_smp_link*
RUN cargo build --release

# Run step
FROM debian:buster-slim as runtime

# Install dependencies
RUN apt update -y && apt install -y default-libmysqlclient-dev && rm -rf /var/lib/apt/lists/*

# Create a folder to recover logs and get .env file
WORKDIR /discord_smp_link/

# Copy the public content
COPY ./public ./public
COPY ./translations ./translations

# Copy the build artifact from the build stage
COPY --from=builder /discord_smp_link/target/release/discord_smp_link ./discord_smp_link

# Set the startup command to run your binary
CMD ["./discord_smp_link"]
