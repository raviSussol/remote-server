use super::InsertSupplierInvoiceError;

pub mod insert;
pub use self::insert::*;

pub mod check_item_id;
pub use self::check_item_id::*;

pub mod check_lines_exist;
pub use self::check_lines_exist::*;

pub mod check_syntax;
pub use self::check_syntax::*;

pub mod helpers;
pub use self::helpers::*;

pub struct SupplierInvoiceLineErrors<T> {
    pub id: String,
    pub errors: Vec<T>,
}

pub type InsertSupplierInvoiceLineErrors =
    SupplierInvoiceLineErrors<InsertSupplierInvoiceLineError>;

pub enum InsertSupplierInvoiceLineError {
    PackSizeMustBeAboveOne(u32),
    SellPricePerPackMustBePositive(f64),
    CostPricePerPackMustBePositive(f64),
    InvoiceLineAlreadyExists,
    ItemIdNotFound(String),
}

impl From<Vec<InsertSupplierInvoiceLineErrors>> for InsertSupplierInvoiceError {
    fn from(errors: Vec<InsertSupplierInvoiceLineErrors>) -> Self {
        InsertSupplierInvoiceError::InvoiceLineErrors(errors)
    }
}

pub fn merge_errors<T>(
    errors: Vec<Vec<SupplierInvoiceLineErrors<T>>>,
) -> Vec<SupplierInvoiceLineErrors<T>> {
    let mut result: Vec<SupplierInvoiceLineErrors<T>> = Vec::new();
    let errors_flattened: Vec<SupplierInvoiceLineErrors<T>> =
        errors.into_iter().flatten().collect();

    for mut error in errors_flattened.into_iter() {
        let matched = result
            .iter_mut()
            .find(|error_to_match| error_to_match.id == error.id);

        if let Some(matched) = matched {
            matched.errors.append(&mut error.errors);
        } else {
            result.push(error);
        }
    }
    result
}
