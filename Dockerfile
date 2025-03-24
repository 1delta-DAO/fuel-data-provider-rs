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
    git \
    curl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Forc (compiler for Sway)
# RUN cargo install forc --locked

# Install fuelup (official toolchain manager for Sway/Fuel)
RUN curl https://install.fuel.network | sh && \
    ln -s $HOME/.fuelup/bin/fuelup /usr/local/bin/fuelup && \
    ln -s $HOME/.fuelup/bin/forc /usr/local/bin/forc && \
    ln -s $HOME/.fuelup/bin/fuel-core /usr/local/bin/fuel-core

# Install custom toolchain using fuelup

RUN fuelup toolchain new 1delta
RUN fuelup default 1delta

RUN fuelup component add fuel-core@0.41.6

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

# Compile Sway contracts to generate ABI files
RUN if [ -d "resources/abi" ]; then \
      echo "Found resources/abi directory, compiling Sway contracts..."; \
      if [ -d "resources/abi/fuel_token_gateway" ]; then \
        cd resources/abi/fuel_token_gateway && \
        echo "Compiling fuel_token_gateway contracts..." && \
        forc clean && \
        forc build && \
        if [ ! -f "out/debug/bridge_fungible_token-abi.json" ]; then \
          echo "ERROR: ABI file not generated at expected path: out/debug/bridge_fungible_token-abi.json" && \
          find out -name "*-abi.json" -type f && \
          if [ -f "out/release/bridge_fungible_token-abi.json" ]; then \
            mkdir -p out/debug && \
            cp out/release/bridge_fungible_token-abi.json out/debug/ && \
            echo "Copied ABI from release to debug directory"; \
          fi; \
        else \
          echo "Successfully generated ABI file at: out/debug/bridge_fungible_token-abi.json"; \
        fi; \
        cd ../../..; \
      fi; \
      for dir in resources/abi/*; do \
        if [ -d "$dir" ] && [ "$(basename "$dir")" != "fuel_token_gateway" ]; then \
          echo "Compiling contracts in $dir..."; \
          cd "$dir" && \
          forc clean && \
          forc build && \
          find out -name "*-abi.json" -type f && \
          cd ../../..; \
        fi; \
      done; \
    else \
      echo "No resources/abi directory found. Skipping Sway compilation."; \
    fi

# List the resources directory to confirm that we have abi files
RUN find resources -type f | sort


RUN echo "--- Forc version and contract build output ---" && \
    forc --version && \
    cd resources/abi/fuel_token_gateway && \
    forc build --debug --print-ast || echo "⚠️ forc build failed" && \
    find out -type f | sort

RUN echo "--- Checking contract structure ---" && \
    ls -l resources/abi/fuel_token_gateway && \
    cat resources/abi/fuel_token_gateway/Forc.toml || echo "No Forc.toml"



RUN test -f resources/abi/fuel_token_gateway/out/debug/bridge_fungible_token-abi.json && \
    echo "ABI file exists at expected path" || \
    echo "WARNING: ABI file does not exist at expected path: resources/abi/fuel_token_gateway/out/debug/bridge_fungible_token-abi.json"

RUN if [ ! -f "resources/abi/fuel_token_gateway/out/debug/bridge_fungible_token-abi.json" ]; then \
      echo "ERROR: ABI file not generated as expected. Stopping build."; \
      exit 1; \
    fi


#RUN if [ ! -f "resources/abi/fuel_token_gateway/out/debug/bridge_fungible_token-abi.json" ]; then \
#      echo "Creating empty ABI file as fallback..."; \
#      mkdir -p resources/abi/fuel_token_gateway/out/debug; \
#      echo '{"types":[],"functions":[]}' > resources/abi/fuel_token_gateway/out/debug/bridge_fungible_token-abi.json; \
#    fi

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

RUN mkdir -p /usr/src/app/migration

COPY --from=builder /usr/src/app/migration /usr/src/app/migration

# Set the working directory for the application
WORKDIR /usr/src/app

# Verify the resources directory in the final image
RUN find resources -type f | sort || echo "Resources directory might be empty"

# Command to run the application
CMD ["fuel_data_provider"]