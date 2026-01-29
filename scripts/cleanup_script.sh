#!/usr/bin/env bash
set -eo pipefail

# [BOOK DIVERGENCE]
# The book doesn't cover automatic database cleanup. This prevents orphaned 
# UUID databases from bloating Docker storage and dragging down Mac performance.

if [ -f .env ]; then
  set -a
  source .env
  set +a
fi

POSTGRES_USER="${POSTGRES_SUPERUSER:-postgres}"
CONTAINER_NAME="newsletter_postgres"

echo "🔍 Scanning for orphaned test databases in ${CONTAINER_NAME}..."

# Using a Heredoc (<<EOF) is the "SecOps" gold standard for piping 
# multi-line SQL into Docker without fighting with shell escaping.
docker exec -i "${CONTAINER_NAME}" psql -U "${POSTGRES_USER}" <<EOF
SELECT 'DROP DATABASE "' || datname || '";' 
FROM pg_database 
WHERE datname ~ '^[0-9a-f-]{36}$' \gexec
EOF

echo "Cleanup complete. CPU and Storage freed."
