#!/usr/bin/env bash
set -eo pipefail

# ---------- Dependency checks ----------
if ! command -v docker &> /dev/null; then
  echo "docker is not installed or not on PATH"
  exit 1
fi

if ! command -v sqlx &> /dev/null; then
  echo "sqlx-cli is not installed"
  echo "Install with: cargo install sqlx-cli --no-default-features --features postgres"
  exit 1
fi

# ---------- Load environment variables ----------
if [ -f .env ]; then
  set -a
  source .env
  set +a
else
  echo ".env file not found"
  exit 1
fi

# ---------- Config ----------
CONTAINER_NAME="newsletter_postgres"

# ---------- Clean up existing container ----------
if docker ps -a --format '{{.Names}}' | grep -Eq "^${CONTAINER_NAME}$"; then
  echo "🧹 Removing existing Postgres container..."
  docker rm -f "${CONTAINER_NAME}"
fi

# ---------- Start Postgres ----------
echo "🚀 Starting Postgres container..."
docker run \
  --env POSTGRES_USER="${POSTGRES_SUPERUSER}" \
  --env POSTGRES_PASSWORD="${POSTGRES_SUPERUSER_PWD}" \
  --publish "${DB_PORT}:5432" \
  --detach \
  --name "${CONTAINER_NAME}" \
  postgres -N 1000

# ---------- Wait for Postgres ----------
echo "Waiting for Postgres to be ready..."
until docker exec "${CONTAINER_NAME}" pg_isready -U "${POSTGRES_SUPERUSER}" &> /dev/null; do
  sleep 1
done

# ---------- Create app role + database ----------
echo "🛠️  Initializing database and role..."
docker exec -i "${CONTAINER_NAME}" psql -U "${POSTGRES_SUPERUSER}" <<EOF
DO \$\$
BEGIN
   IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '${APP_USER}') THEN
      CREATE ROLE ${APP_USER} WITH LOGIN PASSWORD '${APP_USER_PWD}';
   END IF;
END
\$\$;

CREATE DATABASE ${APP_DB} OWNER ${APP_USER};
GRANT ALL PRIVILEGES ON DATABASE ${APP_DB} TO ${APP_USER};
EOF

# ---------- Export DATABASE_URL ----------
export DATABASE_URL

# ---------- Success ----------
echo "Postgres is running!"
echo "   • Host: 127.0.0.1"
echo "   • Port: ${DB_PORT}"
echo "   • Database: ${APP_DB}"
echo "   • User: ${APP_USER}"
