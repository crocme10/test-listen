#!/bin/bash

source ./database.env

PGPASSWORD=${POSTGRES_PASSWORD} psql -h localhost -U "${POSTGRES_USER}" "${POSTGRES_DB}" < build.sql
PGPASSWORD=${POSTGRES_PASSWORD} psql -h localhost -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -c "INSERT INTO ticker VALUES ('AAA', 2.0, default);"
PGPASSWORD=${POSTGRES_PASSWORD} psql -h localhost -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -c "INSERT INTO ticker VALUES ('BBB', 1.0, default);"
PGPASSWORD=${POSTGRES_PASSWORD} psql -h localhost -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -c "INSERT INTO ticker VALUES ('CCC', 4.0, default);"
