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
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Determine architecture and set up symlinks if needed
RUN if [ "$(uname -m)" = "aarch64" ] || [ "$(uname -m)" = "arm64" ]; then \
      if [ ! -d "/usr/lib/aarch64-linux-gnu" ]; then \
        mkdir -p /usr/lib/aarch64-linux-gnu; \
      fi; \
      # Create symlinks for common SSL libraries if they don't exist in the aarch64 dir
      for f in /usr/lib/libssl.so* /usr/lib/libcrypto.so*; do \
        if [ -f "$f" ] && [ ! -f "/usr/lib/aarch64-linux-gnu/$(basename $f)" ]; then \
          ln -sf "$f" "/usr/lib/aarch64-linux-gnu/$(basename $f)"; \
        fi; \
      done; \
    elif [ "$(uname -m)" = "x86_64" ]; then \
      if [ ! -d "/usr/lib/x86_64-linux-gnu" ]; then \
        mkdir -p /usr/lib/x86_64-linux-gnu; \
      fi; \
      # Create symlinks for common SSL libraries if they don't exist in the x86_64 dir
      for f in /usr/lib/libssl.so* /usr/lib/libcrypto.so*; do \
        if [ -f "$f" ] && [ ! -f "/usr/lib/x86_64-linux-gnu/$(basename $f)" ]; then \
          ln -sf "$f" "/usr/lib/x86_64-linux-gnu/$(basename $f)"; \
        fi; \
      done; \
    fi

# Print environment information for debugging
RUN echo "Architecture: $(uname -m)" && \
    find /usr/lib -name "libssl.so*" | sort && \
    find /usr/include -name "openssl" | sort

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create dummy source for dependency caching
RUN mkdir -p src && \
    echo "fn main() {println!(\"Hello World!\");}" > src/main.rs && \
    cargo fetch

# Copy the application source
COPY . .

# List the resources directory to confirm that we have abi files
RUN find resources -type f | sort

# Build the application
RUN cargo build --release

# Stage 2: Create the final image with necessary runtime dependencies
FROM debian:bookworm-slim

# Set environment variables
ENV RUST_BACKTRACE=1

# Install required runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    libssl3 \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage to the final image
COPY --from=builder /usr/src/app/target/release/fuel_data_provider /usr/local/bin/fuel_data_provider

# Create necessary directories
RUN mkdir -p /usr/src/app/resources

# Copy the entire resources directory from builder
COPY --from=builder /usr/src/app/resources /usr/src/app/resources/

# Copy migration files if they exist
COPY --from=builder /usr/src/app/migration /usr/src/app/migration 2>/dev/null || true

# Set the working directory for the application
WORKDIR /usr/src/app

# Verify the resources directory in the final image
RUN find resources -type f | sort || echo "Resources directory might be empty"

# Command to run the application
CMD ["fuel_data_provider"]