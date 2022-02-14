CREATE TYPE changelog_table_name AS ENUM (
    'stocktake'
);

CREATE TYPE row_action_type AS ENUM (
    'UPSERT',
    'DELETE'
);

CREATE TABLE changelog (
    id BIGSERIAL,
    -- the table name where the change happend
    table_name changelog_table_name NOT NULL,
    -- row id of the modified row
    row_id TEXT NOT NULL,
    -- Sqlite only fires INSERT when doing an upsert (it does a delete + insert) for this reason
    -- use UPSERT.
    row_action row_action_type NOT NULL
);

-- View of the changelog that only contains the most recent changes to a row, i.e. previous row
-- edits are removed.
-- Note, an insert + delete will show up as an orphaned delete.
CREATE VIEW changelog_deduped AS
    SELECT t1.id,
        t1.table_name,
        t1.row_id,
        t1.row_action
    FROM changelog t1
    WHERE t1.id = (SELECT max(t2.id) 
                    from changelog t2
                    where t2.row_id = t1.row_id)
    ORDER BY t1.id;

-- Helper trigger function for updating the changelog when a row has been mutated.
-- This function should be used in table triggers.
CREATE OR REPLACE FUNCTION update_changelog()
RETURNS trigger AS
$$
     DECLARE
     BEGIN
        IF (TG_OP = 'DELETE') THEN
            INSERT INTO changelog (table_name, row_id, row_action)
              VALUES (TG_TABLE_NAME::changelog_table_name, OLD.id, 'DELETE');
            RETURN OLD;
        ELSIF (TG_OP = 'UPDATE') THEN
            INSERT INTO changelog (table_name, row_id, row_action)
              VALUES (TG_TABLE_NAME::changelog_table_name, NEW.id, 'UPSERT');
            RETURN NEW;
        ELSIF (TG_OP = 'INSERT') THEN
            INSERT INTO changelog (table_name, row_id, row_action)
              VALUES (TG_TABLE_NAME::changelog_table_name, NEW.id, 'UPSERT');
            RETURN NEW;
        END IF;
     END;
$$ LANGUAGE 'plpgsql';
