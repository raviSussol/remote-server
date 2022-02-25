CREATE TYPE stocktake_status AS ENUM (
    'NEW',
    'FINALISED'
);

CREATE TABLE stocktake (
    id TEXT NOT NULL PRIMARY KEY,
    store_id TEXT NOT NULL REFERENCES store(id),
    stocktake_number BIGINT NOT NULL,
    comment	TEXT,
    description TEXT,
    status stocktake_status NOT NULL,
    created_datetime TIMESTAMP NOT NULL,
    finalised_datetime TIMESTAMP,
    inventory_adjustment_id TEXT REFERENCES invoice(id)
)