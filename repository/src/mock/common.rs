use crate::{
    schema::{
        InvoiceLineRow, InvoiceRow, MasterListLineRow, MasterListNameJoinRow, MasterListRow,
        RequisitionLineRow, RequisitionRow, StockLineRow,
    },
    InvoiceLineRowRepository, InvoiceRepository, MasterListLineRowRepository,
    MasterListNameJoinRepository, MasterListRowRepository, RequisitionLineRowRepository,
    RequisitionRowRepository, StockLineRowRepository, StorageConnection,
};

pub struct FullMockRequisition {
    pub requisition: RequisitionRow,
    pub lines: Vec<RequisitionLineRow>,
}

pub fn insert_full_mock_requisition(
    requisition: &FullMockRequisition,
    connection: &StorageConnection,
) {
    RequisitionRowRepository::new(&connection)
        .upsert_one(&requisition.requisition)
        .unwrap();
    for line in requisition.lines.iter() {
        RequisitionLineRowRepository::new(&connection)
            .upsert_one(line)
            .unwrap();
    }
}

pub struct FullMockInvoiceLine {
    pub line: InvoiceLineRow,
    pub stock_line: StockLineRow,
}

pub struct FullMockInvoice {
    pub invoice: InvoiceRow,
    pub lines: Vec<FullMockInvoiceLine>,
}

impl FullMockInvoice {
    pub fn get_lines(&self) -> Vec<InvoiceLineRow> {
        self.lines
            .iter()
            .map(|full_line| full_line.line.clone())
            .collect()
    }
}

pub fn insert_full_mock_invoice(invoice: &FullMockInvoice, connection: &StorageConnection) {
    InvoiceRepository::new(&connection)
        .upsert_one(&invoice.invoice)
        .unwrap();
    for line in invoice.lines.iter() {
        StockLineRowRepository::new(&connection)
            .upsert_one(&line.stock_line)
            .unwrap();
        InvoiceLineRowRepository::new(&connection)
            .upsert_one(&line.line)
            .unwrap();
    }
}

pub struct FullMockMasterList {
    pub master_list: MasterListRow,
    pub joins: Vec<MasterListNameJoinRow>,
    pub lines: Vec<MasterListLineRow>,
}

pub fn insert_full_mock_master_list(
    full_master_list: &FullMockMasterList,
    connection: &StorageConnection,
) {
    MasterListRowRepository::new(&connection)
        .upsert_one(&full_master_list.master_list)
        .unwrap();

    for line in full_master_list.lines.iter() {
        MasterListLineRowRepository::new(&connection)
            .upsert_one(line)
            .unwrap();
    }

    for join in full_master_list.joins.iter() {
        MasterListNameJoinRepository::new(&connection)
            .upsert_one(join)
            .unwrap();
    }
}
