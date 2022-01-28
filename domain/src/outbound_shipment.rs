use crate::shipment_tax_update::ShipmentTaxUpdate;

use super::invoice::InvoiceStatus;

pub enum UpdateOutboundShipmentStatus {
    Allocated,
    Picked,
    Shipped,
}

pub struct InsertOutboundShipment {
    pub id: String,
    pub other_party_id: String,
    pub status: InvoiceStatus,
    pub on_hold: Option<bool>,
    pub comment: Option<String>,
    pub their_reference: Option<String>,
    pub colour: Option<String>,
}

pub struct UpdateOutboundShipment {
    pub id: String,
    pub other_party_id: Option<String>,
    pub status: Option<UpdateOutboundShipmentStatus>,
    pub on_hold: Option<bool>,
    pub comment: Option<String>,
    pub their_reference: Option<String>,
    pub colour: Option<String>,
}

pub struct InsertOutboundShipmentLine {
    pub id: String,
    pub invoice_id: String,
    pub item_id: String,
    pub stock_line_id: String,
    pub number_of_packs: u32,
    pub total_before_tax: f64,
    pub total_after_tax: f64,
    pub tax: Option<f64>,
}

pub struct UpdateOutboundShipmentLine {
    pub id: String,
    pub invoice_id: String,
    pub item_id: Option<String>,
    pub stock_line_id: Option<String>,
    pub number_of_packs: Option<u32>,
    pub total_before_tax: Option<f64>,
    pub total_after_tax: Option<f64>,
    pub tax: Option<ShipmentTaxUpdate>,
}

pub struct DeleteOutboundShipmentLine {
    pub id: String,
    pub invoice_id: String,
}

impl UpdateOutboundShipmentStatus {
    pub fn full_status(&self) -> InvoiceStatus {
        match self {
            UpdateOutboundShipmentStatus::Allocated => InvoiceStatus::Allocated,
            UpdateOutboundShipmentStatus::Picked => InvoiceStatus::Picked,
            UpdateOutboundShipmentStatus::Shipped => InvoiceStatus::Shipped,
        }
    }
}

impl UpdateOutboundShipment {
    pub fn full_status(&self) -> Option<InvoiceStatus> {
        match &self.status {
            Some(status) => Some(status.full_status()),
            None => None,
        }
    }
}
