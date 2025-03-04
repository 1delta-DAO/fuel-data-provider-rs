# Stage 1: Build the Rust application
FROM rust:slim-bookworm AS builder

# Ustawienie na wersję nightly
RUN rustup default nightly

# Set the working directory inside the container
WORKDIR /usr/src/app

# Install necessary dependencies
RUN apt-get update && \
    apt-get install -y \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    build-essential \
    openssl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Check architecture and set OpenSSL paths accordingly
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then \
        echo "export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu" >> ~/.bashrc && \
        echo "export OPENSSL_INCLUDE_DIR=/usr/include/openssl" >> ~/.bashrc && \
        export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu && \
        export OPENSSL_INCLUDE_DIR=/usr/include/openssl; \
    elif [ "$ARCH" = "arm64" ]; then \
        echo "export OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu" >> ~/.bashrc && \
        echo "export OPENSSL_INCLUDE_DIR=/usr/include/openssl" >> ~/.bashrc && \
        export OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu && \
        export OPENSSL_INCLUDE_DIR=/usr/include/openssl; \
    else \
        echo "export OPENSSL_LIB_DIR=/usr/lib" >> ~/.bashrc && \
        echo "export OPENSSL_INCLUDE_DIR=/usr/include/openssl" >> ~/.bashrc && \
        export OPENSSL_LIB_DIR=/usr/lib && \
        export OPENSSL_INCLUDE_DIR=/usr/include/openssl; \
    fi

# Check architecture and print OpenSSL paths for debugging
RUN ARCH=$(dpkg --print-architecture) && \
    echo "Detected architecture: $ARCH" && \
    if [ "$ARCH" = "amd64" ]; then \
        ls -la /usr/lib/x86_64-linux-gnu/ | grep libssl || echo "libssl not found in expected location"; \
    elif [ "$ARCH" = "arm64" ]; then \
        ls -la /usr/lib/aarch64-linux-gnu/ | grep libssl || echo "libssl not found in expected location"; \
    fi && \
    ls -la /usr/lib/ | grep libssl || echo "libssl not found in /usr/lib/" && \
    ls -la /usr/include/openssl || echo "openssl headers not found"

# Copy the Cargo.toml and Cargo.lock files separately to leverage Docker's layer caching
COPY Cargo.toml Cargo.lock ./

# Ensure rustfmt is installed (optional, useful for formatting)
RUN rustup component add rustfmt

# Pre-fetch the dependencies to speed up the build process
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Copy the source code into the container (including resources and migrations)
COPY . .

# Build with verbose output
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then \
        OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include/openssl RUST_BACKTRACE=1 cargo build --release -vv; \
    elif [ "$ARCH" = "arm64" ]; then \
        OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include/openssl RUST_BACKTRACE=1 cargo build --release -vv; \
    else \
        OPENSSL_LIB_DIR=/usr/lib OPENSSL_INCLUDE_DIR=/usr/include/openssl RUST_BACKTRACE=1 cargo build --release -vv; \
    fi

# Instalacja sea-orm-cli
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then \
        OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include/openssl cargo install sea-orm-cli; \
    elif [ "$ARCH" = "arm64" ]; then \
        OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include/openssl cargo install sea-orm-cli; \
    else \
        OPENSSL_LIB_DIR=/usr/lib OPENSSL_INCLUDE_DIR=/usr/include/openssl cargo install sea-orm-cli; \
    fi

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

# Copy sea-orm-cli from builder stage
COPY --from=builder /usr/local/cargo/bin/sea-orm-cli /usr/local/bin/sea-orm-cli

# Create the resources directory
RUN mkdir -p /usr/src/app/resources

# Copy the configuration file from builder to final image
COPY --from=builder /usr/src/app/resources/config.toml /usr/src/app/resources/

# Copy migration files if they exist
COPY --from=builder /usr/src/app/migration /usr/src/app/migration

# Set the working directory for the application
WORKDIR /usr/src/app

# Command to run the application
CMD ["fuel_data_provider"]