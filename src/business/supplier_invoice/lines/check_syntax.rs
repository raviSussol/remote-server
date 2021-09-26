use crate::{
    business::{InsertSupplierInvoiceLineError, InsertSupplierInvoiceLineErrors},
    server::service::graphql::schema::mutations::supplier_invoice::InsertSupplierInvoiceLineInput,
};

pub fn check_syntax_insert(
    lines: &Vec<InsertSupplierInvoiceLineInput>,
) -> Vec<InsertSupplierInvoiceLineErrors> {
    let mut result = Vec::new();

    for line in lines {
        let errors: Vec<InsertSupplierInvoiceLineError> = check_syntax(
            Some(line.pack_size),
            Some(line.sell_price_per_pack),
            Some(line.cost_price_per_pack),
        )
        .into_iter()
        .map(InsertSupplierInvoiceLineError::from)
        .collect();

        if errors.len() > 0 {
            result.push(InsertSupplierInvoiceLineErrors {
                id: line.id.clone(),
                errors,
            });
        }
    }
    result
}

enum CheckSyntaxError {
    PackSizeMustBeAboveOne(u32),
    SellPricePerPackMustBePositive(f64),
    CostPricePerPackMustBePositive(f64),
}

fn check_syntax(
    pack_size: Option<u32>,
    sell_price_per_pack: Option<f64>,
    cost_price_per_pack: Option<f64>,
) -> Vec<CheckSyntaxError> {
    use self::CheckSyntaxError::*;
    let mut errors = Vec::new();

    if let Some(pack_size) = pack_size {
        if pack_size < 1 {
            errors.push(PackSizeMustBeAboveOne(pack_size))
        }
    }

    if let Some(sell_price_per_pack) = sell_price_per_pack {
        if sell_price_per_pack < 0.0 {
            errors.push(SellPricePerPackMustBePositive(sell_price_per_pack))
        }
    }

    if let Some(cost_price_per_pack) = cost_price_per_pack {
        if cost_price_per_pack < 0.0 {
            errors.push(CostPricePerPackMustBePositive(cost_price_per_pack))
        }
    }

    errors
}

impl From<CheckSyntaxError> for InsertSupplierInvoiceLineError {
    fn from(error: CheckSyntaxError) -> Self {
        use InsertSupplierInvoiceLineError::*;
        match error {
            CheckSyntaxError::PackSizeMustBeAboveOne(pack_size) => {
                PackSizeMustBeAboveOne(pack_size)
            }
            CheckSyntaxError::SellPricePerPackMustBePositive(price) => {
                SellPricePerPackMustBePositive(price)
            }
            CheckSyntaxError::CostPricePerPackMustBePositive(price) => {
                CostPricePerPackMustBePositive(price)
            }
        }
    }
}
