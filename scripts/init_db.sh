#!/usr/bin/env bash
set -eo pipefail

# ---------- Dependency checks ----------
if ! command -v docker &> /dev/null; then
  echo "docker is not installed"
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

# ---------- Validate required env vars ----------
: "${POSTGRES_SUPERUSER:?Missing POSTGRES_SUPERUSER in .env}"
: "${POSTGRES_SUPERUSER_PWD:?Missing POSTGRES_SUPERUSER_PWD in .env}"
: "${APP_USER:?Missing APP_USER in .env}"
: "${APP_USER_PWD:?Missing APP_USER_PWD in .env}"
: "${APP_DB:?Missing APP_DB in .env}"
: "${DB_PORT:?Missing DB_PORT in .env}"

CONTAINER_NAME="newsletter_postgres"

# Override DB_PORT to avoid conflicts with local Postgres if needed
# We use 5435 as your preferred external port
DB_PORT=5435

# ---------- Postgres Container Logic ----------
if [[ -z "${SKIP_DOCKER}" ]]; then
  if docker ps -a --format '{{.Names}}' | grep -Eq "^${CONTAINER_NAME}$"; then
    echo "Removing existing Postgres container..."
    docker rm -f "${CONTAINER_NAME}"
    sleep 2
  fi

  echo "Starting Postgres container on port ${DB_PORT}..."
  # Note: Internal port is 5432 (default Postgres), External is 5435
  docker run \
    --env POSTGRES_USER="${POSTGRES_SUPERUSER}" \
    --env POSTGRES_PASSWORD="${POSTGRES_SUPERUSER_PWD}" \
    --publish "127.0.0.1:${DB_PORT}:5432" \
    --detach \
    --name "${CONTAINER_NAME}" \
    postgres -N 1000

  echo "Waiting for Postgres to be ready..."
  # Added -h localhost to force TCP instead of Unix Socket
  until docker exec "${CONTAINER_NAME}" pg_isready -h localhost -U "${POSTGRES_SUPERUSER}" &> /dev/null; do
    sleep 1
  done

  # ---------- Step 1: Create the User ----------
  echo "Creating app user: ${APP_USER}"
  # Added -h localhost to all psql commands
  docker exec -i "${CONTAINER_NAME}" psql -h localhost -U "${POSTGRES_SUPERUSER}" <<EOF
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = '${APP_USER}') THEN
        CREATE ROLE ${APP_USER} WITH LOGIN PASSWORD '${APP_USER_PWD}';
    END IF;
END
\$\$;
ALTER USER ${APP_USER} CREATEDB;
EOF

  # ---------- Step 2: Create the Database ----------
  echo "Creating database: ${APP_DB}"
  docker exec -i "${CONTAINER_NAME}" psql -h localhost -U "${POSTGRES_SUPERUSER}" <<EOF
SELECT 'CREATE DATABASE ${APP_DB}'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '${APP_DB}')\gexec
ALTER DATABASE ${APP_DB} OWNER TO ${APP_USER};
GRANT ALL PRIVILEGES ON DATABASE ${APP_DB} TO ${APP_USER};
EOF

  # Verify role exists in container
  echo "Verifying role '${APP_USER}' exists in container..."
  ROLE_CHECK=$(docker exec "${CONTAINER_NAME}" psql -h localhost -U "${POSTGRES_SUPERUSER}" -tAc "SELECT 1 FROM pg_roles WHERE rolname='${APP_USER}';")
  if [[ "${ROLE_CHECK}" != "1" ]]; then
    echo "Error: Role '${APP_USER}' was not created successfully"
    exit 1
  fi
  
  # Test connection from inside the container via TCP
  echo "Testing connection to database '${APP_DB}' with user '${APP_USER}'..."
  if ! docker exec "${CONTAINER_NAME}" psql -h localhost -U "${APP_USER}" -d "${APP_DB}" -c "SELECT 1;" &>/dev/null; then
    echo "Error: Cannot connect with app user credentials from inside container"
    exit 1
  fi
fi

# ---------- Step 3: Run Migrations ----------
# Using 127.0.0.1 instead of localhost to bypass macOS IPv6 resolution issues
export DATABASE_URL="postgres://${APP_USER}:${APP_USER_PWD}@127.0.0.1:${DB_PORT}/${APP_DB}?sslmode=disable"

echo "Running migrations with URL: ${DATABASE_URL}"

# Give the network bridge a moment to stabilize
sleep 2

sqlx migrate run