use super::invoice::InvoiceStatus;

pub struct InsertOutboundShipment {
    pub id: String,
    pub other_party_id: String,
    pub status: InvoiceStatus,
    pub on_hold: Option<bool>,
    pub comment: Option<String>,
    pub their_reference: Option<String>,
}

pub struct UpdateOutboundShipment {
    pub id: String,
    pub other_party_id: Option<String>,
    pub status: Option<InvoiceStatus>,
    pub on_hold: Option<bool>,
    pub comment: Option<String>,
    pub their_reference: Option<String>,
}

pub struct InsertOutboundShipmentLine {
    pub id: String,
    pub invoice_id: String,
    pub item_id: String,
    pub stock_line_id: String,
    pub number_of_packs: u32,
}

pub struct UpdateOutboundShipmentLine {
    pub id: String,
    pub invoice_id: String,
    pub item_id: Option<String>,
    pub stock_line_id: Option<String>,
    pub number_of_packs: Option<u32>,
}

pub struct DeleteOutboundShipmentLine {
    pub id: String,
    pub invoice_id: String,
}