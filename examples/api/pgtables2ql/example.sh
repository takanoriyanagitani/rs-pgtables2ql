#!/bin/sh


export DATABASE_URL=postgresql://localhost:5433/postgres
export LISTEN_ADDR=127.0.0.1:7258

./pgtables2ql
