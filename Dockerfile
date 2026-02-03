# Use latest stable rust release as base img
FROM rust:1.91.1

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

ENV APP_ENVIRONMENT=production

# Once built, let's launch this puppy
ENTRYPOINT [ "./target/release/newsletter-service" ]
