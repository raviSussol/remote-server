-- Create requisition_line table.

CREATE TABLE requisition_line (
    id TEXT NOT NULL PRIMARY KEY,
    requisition_id TEXT NOT NULL REFERENCES requisition (id),
    item_id TEXT NOT NULL,
    requested_quantity INTEGER NOT NULL,
    suggested_quantity INTEGER NOT NULL,
    supply_quantity INTEGER NOT NULL,
    available_stock_on_hand INTEGER NOT NULL,
    average_monthly_consumption INTEGER NOT NULL
)
