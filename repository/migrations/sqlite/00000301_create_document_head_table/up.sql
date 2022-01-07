CREATE TABLE document_head (
    id TEXT NOT NULL PRIMARY KEY,
    store TEXT NOT NULL,
    name TEXT NOT NULL,
    head TEXT NOT NULL REFERENCES document(id)
)