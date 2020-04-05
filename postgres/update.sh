#!/bin/bash

if [ "$#" -ne 2 ]; then
  echo "Usage: ./update.sh [ticker] [price]"
  exit 1
fi

source ./database.env

PGPASSWORD=${POSTGRES_PASSWORD} psql -h localhost -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -c "UPDATE ticker SET price=$2 WHERE id='$1';"
