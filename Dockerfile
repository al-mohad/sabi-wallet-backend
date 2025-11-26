# Dockerfile for Sabi Wallet Backend

# ---- Builder Stage ----
# Use the official Rust image as a build environment.
# We use the bookworm-slim variant for a smaller base image.
FROM rust:1.78-bookworm-slim as builder

# Set the working directory
WORKDIR /app

# Install dependencies needed for building
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    # Add any other build-time dependencies here
    && rm -rf /var/lib/apt/lists/*

# Copy the Cargo files to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy `src/main.rs` to build only the dependencies.
# This layer is cached and will only be rebuilt if Cargo.toml/Cargo.lock changes.
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --locked

# Remove the dummy source and copy the actual source code
RUN rm -rf src
COPY src ./

# Install sqlx-cli for migrations (optional, but good practice)
RUN cargo install sqlx-cli --version="0.7.4" --features="rustls,postgres,sqlite" --no-default-features --locked

# Build the application, this will re-use the cached dependency layers
RUN touch src/main.rs && cargo build --release --locked

# ---- Final Stage ----
# Use a minimal base image for the final container
# Debian slim is a good choice for a balance of size and utility (like having a shell)
FROM debian:bookworm-slim as final

# Set the working directory
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/sabi-wallet-backend .

# Copy migrations and .env file
# Note: In a real production setup, you might not copy .env and use a secret management system instead.
COPY --from=builder /root/.cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations ./migrations
COPY .env .

# Expose the port the application will run on
EXPOSE 8080

# Set the entrypoint for the container.
# This will run migrations and then start the application.
CMD ["sh", "-c", "sqlx migrate run && ./sabi-wallet-backend"]
