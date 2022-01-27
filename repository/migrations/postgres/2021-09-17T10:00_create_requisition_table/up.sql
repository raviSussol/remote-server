-- Create requisition table.

CREATE TYPE requisition_type AS ENUM ('REQUEST', 'RESPONSE');
CREATE TYPE requisition_status AS ENUM ('DRAFT', 'NEW', 'SENT', 'FINALISED');

CREATE TABLE requisition (
    id TEXT NOT NULL PRIMARY KEY,
    requisition_number BIGINT NOT NULL,
    store_id TEXT NOT NULL REFERENCES name(id),
    name_id TEXT NOT NULL REFERENCES store(id),
    type requisition_type NOT NULL,
    status requisition_status NOT NULL,
    created_datetime TIMESTAMP NOT NULL,
    sent_datetime TIMESTAMP,
    finalised_datetime TIMESTAMP,
    color TEXT,
    comment TEXT,
    their_reference TEXT,
    max_months_of_stock  DOUBLE PRECISION NOT NULL,
    threshold_months_of_stock DOUBLE PRECISION NOT NULL,
    linked_requisition_id TEXT,
    FOREIGN KEY (linked_requisition_id) REFERENCES requisition(id)
)