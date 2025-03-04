# Stage 1: Build the Rust application
FROM rust:slim-bookworm AS builder

# Ustawienie na wersję nightly
RUN rustup default nightly

# Set the working directory inside the container
WORKDIR /usr/src/app

# Install necessary dependencies with specific OpenSSL paths
RUN apt-get update && \
    apt-get install -y \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    build-essential \
    openssl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy the Cargo.toml and Cargo.lock files separately to leverage Docker's layer caching
COPY Cargo.toml Cargo.lock ./

# Ensure rustfmt is installed (optional, useful for formatting)
RUN rustup component add rustfmt

# Pre-fetch the dependencies to speed up the build process
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build the application in release mode with explicit environment variables for OpenSSL
ENV OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl
RUN cargo fetch

# Copy the source code into the container (including resources)
COPY . .

# Build with verbose output
RUN RUST_BACKTRACE=1 cargo build --release

# Stage 2: Create the final image with necessary runtime dependencies
FROM debian:bookworm-slim

# Set environment variables
ENV RUST_BACKTRACE=1

# Install required runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    curl \
    libssl3 \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage to the final image
COPY --from=builder /usr/src/app/target/release/fuel_data_provider /usr/local/bin/fuel_data_provider

# Create the resources directory
RUN mkdir -p /usr/src/app/resources

# Copy the configuration file from builder to final image
COPY --from=builder /usr/src/app/resources/config.toml /usr/src/app/resources/

# Set the working directory for the application
WORKDIR /usr/src/app

# Command to run the application
CMD ["fuel_data_provider"]