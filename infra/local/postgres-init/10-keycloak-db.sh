#!/bin/sh
# Init lokal untuk klaster aplikasi RS: membuat role dan database `keycloak`
# (docs/05-stack.md: instans (a) memuat DB aplikasi Ruang Sehat + DB keycloak).
# Hanya dijalankan docker-entrypoint pada inisialisasi pertama volume kosong.
# Password dari variabel lingkungan, tidak ditulis di berkas ini. Role aplikasi lain: F1-T13.
set -eu
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname postgres \
  -v kcpass="$KEYCLOAK_DB_PASSWORD" <<'SQL'
CREATE ROLE keycloak LOGIN PASSWORD :'kcpass';
CREATE DATABASE keycloak OWNER keycloak;
SQL
