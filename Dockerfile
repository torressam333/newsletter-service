# Use latest stable rust release as base img
FROM rust:1.91.1 AS builder

# Switch the working dir to app, Docker will create it if it doesnt exist
WORKDIR /app

# Install reqd deps for linking config
RUN apt update && apt install lld clang -y

# Copy all files from workiing dir to Docker Image
COPY . .

# Tell Docker to looko at saved json metadata...don't need a live DB
ENV SQLX_OFFLINE=true

# Build the binary
RUN cargo build --release

#RUNTIME STAGE
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
