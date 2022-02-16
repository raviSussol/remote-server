CREATE TYPE key_type AS ENUM (
    -- Cursor for pulling central records from the central server
    'CENTRAL_SYNC_PULL_CURSOR',
    'REMOTE_SYNC_QUEUE_V5_INITALISED',
    'REMOTE_SYNC_INITIAL_SYNC_STATE'
);

-- key value store, e.g. to store local server state
CREATE TABLE key_value_store (
    id key_type NOT NULL PRIMARY KEY,
    value_string TEXT
)