use crate::{
    business::{InsertSupplierInvoiceLineError, InsertSupplierInvoiceLineErrors},
    server::service::graphql::schema::mutations::supplier_invoice::{
        InsertSupplierInvoiceLineInput, UpsertSupplierInvoiceLineInput,
    },
};

use super::{UpsertSupplierInvoiceLineError, UpsertSupplierInvoiceLineErrors};

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

pub fn check_syntax_upsert(
    lines: &Vec<UpsertSupplierInvoiceLineInput>,
) -> Vec<UpsertSupplierInvoiceLineErrors> {
    let mut result = Vec::new();

    for line in lines {
        let errors: Vec<UpsertSupplierInvoiceLineError> = check_syntax(
            line.pack_size,
            line.sell_price_per_pack,
            line.cost_price_per_pack,
        )
        .into_iter()
        .map(UpsertSupplierInvoiceLineError::from)
        .collect();

        if errors.len() > 0 {
            result.push(UpsertSupplierInvoiceLineErrors {
                id: line.id.clone(),
                errors,
            });
        }
    }
    result
}

pub struct InsertsFromUpsert {
    pub inserts: Vec<InsertSupplierInvoiceLineInput>,
    pub errors: Vec<UpsertSupplierInvoiceLineErrors>,
}

fn get_insert_form_upsert(
    UpsertSupplierInvoiceLineInput {
        id,
        pack_size,
        batch,
        number_of_packs,
        item_id,
        cost_price_per_pack,
        sell_price_per_pack,
        expiry_date,
    }: UpsertSupplierInvoiceLineInput,
) -> Result<InsertSupplierInvoiceLineInput, UpsertSupplierInvoiceLineError> {
    use super::RequiredInsertField::*;
    use UpsertSupplierInvoiceLineError::*;

    Ok(InsertSupplierInvoiceLineInput {
        id,
        pack_size: pack_size.ok_or(InsertFieldMissing(PackSize))?,
        batch,
        number_of_packs: number_of_packs.ok_or(InsertFieldMissing(NumberOfPacks))?,
        item_id: item_id.ok_or(InsertFieldMissing(ItemId))?,
        cost_price_per_pack: cost_price_per_pack.ok_or(InsertFieldMissing(CostPricePerPack))?,
        sell_price_per_pack: sell_price_per_pack.ok_or(InsertFieldMissing(SellPricePerPack))?,
        expiry_date,
    })
}

pub fn get_inserts_from_upsert(lines: Vec<UpsertSupplierInvoiceLineInput>) -> InsertsFromUpsert {
    let mut inserts = Vec::new();
    let mut errors = Vec::new();

    for line in lines.into_iter() {
        let id = line.id.clone();
        match get_insert_form_upsert(line) {
            Ok(insert) => inserts.push(insert),
            Err(error) => errors.push(UpsertSupplierInvoiceLineErrors {
                id,
                errors: vec![error],
            }),
        }
    }

    InsertsFromUpsert { inserts, errors }
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

impl From<CheckSyntaxError> for UpsertSupplierInvoiceLineError {
    fn from(error: CheckSyntaxError) -> Self {
        use UpsertSupplierInvoiceLineError::*;
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
