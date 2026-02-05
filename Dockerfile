FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

# PLAN STAGE
FROM chef AS planner
COPY . .

# Compute a lock like file for our proj
RUN cargo chef prepare --recipe-path recipe.json

# BUILD STAGE
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE=true

# Build application
RUN cargo build --release --bin newsletter-service

#RUNTIME STAGE We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime

# Install OpenSSL, ca certs etc..
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the biniary from builder env above to runtime env
COPY --from=builder /app/target/release/newsletter-service newsletter-service
COPY configuration configuration

ENV APP_ENVIRONMENT=production

# Once built, let's launch this puppy
ENTRYPOINT [ "./newsletter-service" ]
