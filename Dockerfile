# Build layer
FROM rustlang/rust:nightly as builder

# Add LLVM project APT repository
RUN wget https://apt.llvm.org/llvm-snapshot.gpg.key
RUN apt-key add llvm-snapshot.gpg.key
RUN rm llvm-snapshot.gpg.key

# Install developement libraries and headers
RUN apt update -y
RUN apt install -y default-libmysqlclient-dev clang lldb lld
RUN apt clean -y

# Create a new empty shell project
RUN USER=root cargo new --bin discord_smp_link
WORKDIR /discord_smp_link/

COPY ./.cargo ./.cargo
# Copy over the manifests
COPY Cargo.toml Cargo.lock ./

# Build and cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy the source
COPY ./src ./src
COPY ./templates ./templates
COPY ./migrations ./migrations

# Build for release
RUN rm ./target/release/deps/discord_smp_link*
RUN cargo build --release

# Run step
FROM debian:buster-slim as runtime

# Install dependencies
RUN apt update -y
RUN apt install -y default-libmysqlclient-dev libssl-dev ca-certificates
RUN rm -rf /var/lib/apt/lists/*

# Create a folder to recover logs and get .env file
WORKDIR /discord_smp_link/

# Copy the public content
COPY ./public ./public
COPY ./translations ./translations

# Copy the build artifact from the build stage
COPY --from=builder /discord_smp_link/target/release/discord_smp_link ./discord_smp_link

# Set the startup command to run your binary
CMD ["./discord_smp_link"]
