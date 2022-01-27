CREATE TYPE invoice_type AS ENUM (
    'OUTBOUND_SHIPMENT',
    'INBOUND_SHIPMENT',
    'INVENTORY_ADJUSTMENT'
);

CREATE TYPE invoice_status AS ENUM (
    'NEW',
    'ALLOCATED',
    'PICKED',
    'SHIPPED',
    'DELIVERED',
    'VERIFIED'
);

CREATE TABLE invoice (
    id TEXT NOT NULL PRIMARY KEY,
    -- For outbound shipments, the id of the receiving customer.
    -- For inbound shipments, the id of the sending supplier.
    name_id TEXT NOT NULL REFERENCES name(id),
    name_store_id TEXT REFERENCES store (id),
    -- For outbound shipments, the id of the issuing store.
    -- For inbound shipments, the id of the receiving store.
    store_id TEXT NOT NULL REFERENCES store (id),
    invoice_number BIGINT NOT NULL,
    type invoice_type NOT NULL,
    status invoice_status NOT NULL,
    on_hold BOOLEAN NOT NULL,
    comment TEXT,
    their_reference TEXT,
    created_datetime TIMESTAMP NOT NULL,
    allocated_datetime TIMESTAMP,
    picked_datetime TIMESTAMP,
    shipped_datetime TIMESTAMP,
    delivered_datetime TIMESTAMP,
    verified_datetime TIMESTAMP,
    color TEXT,
    requisition_id TEXT REFERENCES requisition(id),
    linked_invoice_id TEXT,
    FOREIGN KEY (linked_invoice_id) REFERENCES invoice(id)    
)