-- Create store table.

CREATE TABLE store (
    id TEXT NOT NULL PRIMARY KEY,
    name_id TEXT NOT NULL,
    code TEXT NOT NULL,
    FOREIGN KEY(name_id) REFERENCES name(id)
)
