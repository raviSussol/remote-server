use crate::invoice::{
    check_invoice_exists, check_invoice_is_editable, check_invoice_status, check_invoice_type,
    check_other_party_id, InvoiceDoesNotExist, InvoiceIsNotEditable, InvoiceStatusError,
    WrongInvoiceType,
};
use domain::{inbound_shipment::UpdateInboundShipment, invoice::InvoiceType, name::Name};
use repository::{schema::InvoiceRow, StorageConnection};

use super::UpdateInboundShipmentError;

pub fn validate(
    patch: &UpdateInboundShipment,
    connection: &StorageConnection,
) -> Result<(InvoiceRow, Option<Name>), UpdateInboundShipmentError> {
    use UpdateInboundShipmentError::*;
    let invoice = check_invoice_exists(&patch.id, connection)?;

    // check_store(invoice, connection)?; InvoiceDoesNotBelongToCurrentStore
    check_invoice_type(&invoice, InvoiceType::InboundShipment)?;
    check_invoice_is_editable(&invoice)?;
    check_invoice_status(&invoice, patch.full_status(), &patch.on_hold)?;

    let other_party_option = match &patch.other_party_id {
        Some(other_party_id) => {
            let other_party = check_other_party_id(connection, &other_party_id)?
                .ok_or(OtherPartyDoesNotExist {})?;

            if !other_party.is_supplier {
                return Err(OtherPartyNotASupplier(other_party));
            };
            Some(other_party)
        }
        None => None,
    };

    Ok((invoice, other_party_option))
}

impl From<WrongInvoiceType> for UpdateInboundShipmentError {
    fn from(_: WrongInvoiceType) -> Self {
        UpdateInboundShipmentError::NotAnInboundShipment
    }
}

impl From<InvoiceIsNotEditable> for UpdateInboundShipmentError {
    fn from(_: InvoiceIsNotEditable) -> Self {
        UpdateInboundShipmentError::CannotEditFinalised
    }
}

impl From<InvoiceDoesNotExist> for UpdateInboundShipmentError {
    fn from(_: InvoiceDoesNotExist) -> Self {
        UpdateInboundShipmentError::InvoiceDoesNotExist
    }
}

impl From<InvoiceStatusError> for UpdateInboundShipmentError {
    fn from(error: InvoiceStatusError) -> Self {
        use UpdateInboundShipmentError::*;
        match error {
            InvoiceStatusError::CannotChangeStatusOfInvoiceOnHold => {
                CannotChangeStatusOfInvoiceOnHold
            }
            InvoiceStatusError::CannotReverseInvoiceStatus => CannotReverseInvoiceStatus,
        }
    }
}
