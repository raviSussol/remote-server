#!/bin/bash

set -e

cd /usr/src/remote-server

service postgresql start

sleep 5

psql -U postgres -c 'CREATE DATABASE "omsupply-database"'

export DATABASE_URL=postgres://postgres@localhost:5432/omsupply-database

sqlx migrate --source migrations/pg run
cargo sqlx prepare -- --lib

cargo build