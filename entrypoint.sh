#!/bin/sh

set -e
echo "--- IMPRIMIENDO VARIABLES DE ENTORNO ---"
printenv
echo "-----------------------------------------"

# Wait for Postgres
echo "Esperando a Postgres en ${POSTGRES_HOST}:${POSTGRES_PORT}..."
until PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c '\q'; do
  >&2 echo "Postgres no está disponible - durmiendo"
  sleep 1
done
>&2 echo "Postgres está listo - ejecutando migraciones."

# Run migrations
sqlx migrate run || echo "Error al ejecutar migraciones."

# Wait for TigerBeetle
echo "Esperando a TigerBeetle en ${TIGERBEETLE_HOST}:${TIGERBEETLE_PORT}..."
until nc -z -v "$TIGERBEETLE_HOST" "$TIGERBEETLE_PORT"; do
  >&2 echo "TigerBeetle no está disponible - durmiendo"
  sleep 1
done
>&2 echo "TigerBeetle está listo - ejecutando script de inicialización."

# Keep container running for debugging
echo "Iniciando la aplicación..."
exec ./main
