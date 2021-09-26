use chrono::{NaiveDateTime, Utc};

use crate::{
    database::{
        repository::RepositoryError,
        schema::{InvoiceLineRow, InvoiceRow, StockLineRow},
    },
    server::service::graphql::schema::types::NameQuery,
};

pub mod check_invoice;
pub use self::check_invoice::*;

pub mod check_other_party;
pub use self::check_other_party::*;

pub mod check_store;
pub use self::check_store::*;

pub mod insert;
pub use self::insert::*;

pub mod update;
pub use self::update::*;

pub mod lines;
pub use self::lines::*;

pub struct FullInvoice {
    pub invoice: InvoiceRow,
    pub lines: Vec<FullInvoiceLine>,
}
pub struct FullInvoiceLine {
    pub line: InvoiceLineRow,
    pub batch: Option<StockLineRow>,
}

pub struct Mutations<T> {
    pub inserts: Option<Vec<T>>,
    pub updates: Option<Vec<T>>,
    pub deletes: Option<Vec<T>>,
}

impl<T> Mutations<T> {
    fn new() -> Mutations<T> {
        Mutations {
            inserts: None,
            updates: None,
            deletes: None,
        }
    }
    fn new_inserts(value: T) -> Mutations<T> {
        Mutations {
            inserts: Some(vec![value]),
            updates: None,
            deletes: None,
        }
    }
    fn new_updates(value: T) -> Mutations<T> {
        Mutations {
            inserts: None,
            updates: Some(vec![value]),
            deletes: None,
        }
    }
    fn new_deletes(value: T) -> Mutations<T> {
        Mutations {
            inserts: None,
            updates: None,
            deletes: Some(vec![value]),
        }
    }
    fn add_insert(&mut self, value: T) -> &Self {
        add_to_mutations(&mut self.inserts, value);
        self
    }

    fn add_update(&mut self, value: T) -> &Self {
        add_to_mutations(&mut self.updates, value);
        self
    }

    fn add_delete(&mut self, value: T) -> &Self {
        add_to_mutations(&mut self.deletes, value);
        self
    }
}

fn add_to_mutations<T>(mutations: &mut Option<Vec<T>>, value: T) {
    match mutations {
        Some(mutations) => mutations.push(value),
        None => *mutations = Some(vec![value]),
    };
}

pub struct FullInvoiceMutation {
    pub invoice: Mutations<InvoiceRow>,
    pub lines: Mutations<InvoiceLineRow>,
    pub batches: Mutations<StockLineRow>,
}

pub enum InsertSupplierInvoiceError {
    OtherPartyNotFound(String),
    OtherPartyIsNotASupplier(NameQuery),
    InvoiceExists,
    InvoiceLineErrors(Vec<InsertSupplierInvoiceLineErrors>),
    DBError(RepositoryError),
}

pub struct InsertSupplierInvoiceLineErrors {
    pub id: String,
    pub errors: Vec<InsertSupplierInvoiceLineError>,
}
pub enum InsertSupplierInvoiceLineError {
    PackSizeMustBeAboveOne(u32),
    SellPricePerPackMustBePositive(f64),
    CostPricePerPackMustBePositive(f64),
    InvoiceLineAlreadyExists,
    ItemIdNotFound(String),
}

pub enum UpdateSupplierInvoiceError {
    OtherPartyNotFound(String),
    OtherPartyIsNotASupplier(NameQuery),
    CannotEditFinalisedInvoice,
    InvoiceDoesNotExist,
    NotASupplierInvoice,
    InvoiceDoesNotBelongToCurrentStore,
    CannoChangeInvoiceBackToDraft,
    DBError(RepositoryError),
}

pub fn current_date_time() -> NaiveDateTime {
    Utc::now().naive_utc()
}
