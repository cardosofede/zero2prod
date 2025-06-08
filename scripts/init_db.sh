#!/usr/bin/env bash
set -x
set -eo pipefail

DB_PORT=${DB_PORT:-5432}
SUPERUSER=${SUPERUSER:-postgres}
SUPERUSER_PWD=${SUPERUSER_PWD:-password}
CONTAINER_NAME=${CONTAINER_NAME:-postgres}

APP_USER=${APP_USER:-app}
APP_USER_PWD=${APP_USER_PWD:-secret}
APP_DB_NAME=${APP_DB_NAME:-newsletter}

if [[ -z $SKIP_DOCKER ]]
then
  docker run --rm -d \
    --name "$CONTAINER_NAME" \
    -e POSTGRES_USER="$SUPERUSER" \
    -e POSTGRES_PASSWORD="$SUPERUSER_PWD" \
    -p "$DB_PORT":5432 \
    postgres:latest -N 1000

  # Wait for PostgreSQL to be ready
  until docker exec "$CONTAINER_NAME" pg_isready -U "$SUPERUSER" -h localhost -p 5432; do
    >&2 echo "PostgreSQL is starting up..."
    sleep 1
  done

  >&2 echo "PostgreSQL is ready."

  CREATE_QUERY="CREATE USER $APP_USER WITH PASSWORD '$APP_USER_PWD';"
  docker exec -i "$CONTAINER_NAME" psql -U "$SUPERUSER" -c "$CREATE_QUERY"

  GRANT_QUERY="ALTER USER ${APP_USER} CREATEDB;"
  docker exec -i "$CONTAINER_NAME" psql -U "$SUPERUSER" -c "$GRANT_QUERY"
fi

DATABASE_URL="postgresql://${APP_USER}:${APP_USER_PWD}@localhost:${DB_PORT}/${APP_DB_NAME}"
export DATABASE_URL
sqlx database create
sqlx migrate run