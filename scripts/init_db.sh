#!/usr/bin/env bash
set -x
set -eo pipefail

# -----------------------------
# Load environment variables from .env if present
# -----------------------------
if [ -f .env ]; then
  set -a   # automatically export all variables
  source .env
  set +a
fi

# -----------------------------
# Default values if not set
# -----------------------------
DB_PORT="${DB_PORT:=5432}"

POSTGRES_SUPERUSER="${POSTGRES_SUPERUSER:=postgres}"
POSTGRES_SUPERUSER_PWD="${POSTGRES_SUPERUSER_PWD:=password}"

APP_USER="${APP_USER:=app}"
APP_USER_PWD="${APP_USER_PWD:=secret}"
APP_DB="${APP_DB:=newsletter}"

CONTAINER_NAME="newsletter_postgres"

# -----------------------------
# Cleanup existing container if present
# -----------------------------
if docker ps -a --format '{{.Names}}' | grep -Eq "^${CONTAINER_NAME}\$"; then
  docker rm -f "${CONTAINER_NAME}"
fi

# -----------------------------
# Start Postgres container
# -----------------------------
docker run \
  --env POSTGRES_USER="${POSTGRES_SUPERUSER}" \
  --env POSTGRES_PASSWORD="${POSTGRES_SUPERUSER_PWD}" \
  --publish "${DB_PORT}:5432" \
  --detach \
  --name "${CONTAINER_NAME}" \
  postgres -N 1000

# -----------------------------
# Wait until Postgres is ready
# -----------------------------
until docker exec "${CONTAINER_NAME}" pg_isready -U "${POSTGRES_SUPERUSER}" > /dev/null 2>&1; do
  sleep 1
done

# -----------------------------
# Bootstrap app user and database
# -----------------------------
docker exec -i "${CONTAINER_NAME}" psql -U "${POSTGRES_SUPERUSER}" <<EOF
CREATE ROLE ${APP_USER} WITH LOGIN PASSWORD '${APP_USER_PWD}' CREATEDB;
CREATE DATABASE ${APP_DB} OWNER ${APP_USER};
GRANT ALL PRIVILEGES ON DATABASE ${APP_DB} TO ${APP_USER};
EOF

# -----------------------------
# Success message
# -----------------------------
echo "Postgres is up and running!"
echo "   → Port: ${DB_PORT}"
echo "   → Database: ${APP_DB}"
echo "   → App user: ${APP_USER}"
echo "   → Connection URL:"
echo "     postgres://${APP_USER}:${APP_USER_PWD}@127.0.0.1:${DB_PORT}/${APP_DB}"
