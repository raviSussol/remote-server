use std::collections::BTreeMap;

use std::time::Duration;

use crate::database::repository::{
    ItemLineRepository, ItemRepository, RequisitionRepository, StoreRepository,
    TransactLineRepository, TransactRepository,
};
use crate::database::schema::{
    ItemLineRow, ItemRow, NameRow, RequisitionRow, StoreRow, TransactLineRow, TransactRow,
};
use crate::server::service::graphql::schema::types::{
    Item, ItemLine, Requisition, Store, Transact, TransactLine,
};
use crate::server::service::graphql::ContextExt;

use actix_rt::time::delay_for;
use async_graphql::{
    Context, Enum, Error, ErrorExtensions, InputObject, Name as AGname, Object, Value,
};

use super::types::Name;

pub struct Queries;

struct ErrorDetailsExample {
    id: String,
    value: u16,
}

struct ErrorDetailsExample2 {
    value: String,
    id: u16,
}

enum ErrorExample {
    Panic,
    BasicString(String),
    StructuredError,
}

impl From<ErrorDetailsExample> for Value {
    fn from(detail_example: ErrorDetailsExample) -> Self {
        let mut map: BTreeMap<AGname, Value> = BTreeMap::new();

        map.insert(
            AGname::new("id"),
            Value::String(detail_example.id.to_owned()),
        );
        map.insert(
            AGname::new("value"),
            Value::Number(detail_example.value.into()),
        );
        Value::Object(map)
    }
}

impl From<ErrorDetailsExample2> for Value {
    fn from(detail_example: ErrorDetailsExample2) -> Self {
        let mut map: BTreeMap<AGname, Value> = BTreeMap::new();

        map.insert(
            AGname::new("value"),
            Value::String(detail_example.value.to_owned()),
        );
        map.insert(AGname::new("id"), Value::Number(detail_example.id.into()));
        Value::Object(map)
    }
}

impl ErrorExtensions for ErrorExample {
    fn extend(&self) -> Error {
        match self {
            ErrorExample::Panic => Error::new(format!("Panice, this will never show"))
                .extend_with(|_, _| panic!("something went wrong")), // this wouldn't be in error, it would vai panic in logic that we've missed
            ErrorExample::BasicString(value) => Error::new(format!("String Error"))
                .extend_with(|_, e| e.set("stringError", value.to_owned())),
            ErrorExample::StructuredError => {
                Error::new(format!("Structured Error")).extend_with(|_, e| {
                    e.set(
                        "arrayOfErrors",
                        vec![
                            ErrorDetailsExample {
                                id: "first id".to_owned(),
                                value: 1,
                            },
                            ErrorDetailsExample {
                                id: "second id".to_owned(),
                                value: 2,
                            },
                        ],
                    );
                    e.set(
                        "errorObject",
                        ErrorDetailsExample2 {
                            value: "value".to_owned(),
                            id: 1,
                        },
                    );
                })
            }
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ExampleInputVariants {
    ONE,
    TWO,
    THREE,
}

#[derive(InputObject)]
pub struct ExampleInput {
    r#type: ExampleInputVariants,
    direction: Option<bool>,
}

#[Object]
impl Queries {
    #[allow(non_snake_case)]
    pub async fn apiVersion(&self) -> String {
        "1.0".to_string()
    }

    pub async fn name(
        &self,
        _ctx: &Context<'_>,
        #[graphql(desc = "id of the name")] id: String,
        delay_seconds: Option<u64>,
        _structured_input: Option<Vec<ExampleInput>>,
    ) -> Result<Name, Error> {
        if let Some(delay_seconds) = delay_seconds {
            delay_for(Duration::new(delay_seconds, 0)).await;
        }

        match &id[..] {
            "this_causes_uhandled_panic" => return Err(ErrorExample::Panic.extend()),
            "this_causes_string_error" => {
                return Err(
                    ErrorExample::BasicString(format!("Some error string, from id {}", id))
                        .extend(),
                )
            }
            "this_causes_structured_error" => return Err(ErrorExample::StructuredError.extend()),

            _ => {}
        };

        Ok(Name {
            name_row: NameRow {
                id: "TEST".to_owned(),
                name: "TEST".to_owned(),
            },
        })
    }

    pub async fn store(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the store")] id: String,
    ) -> Store {
        let store_repository = ctx.get_repository::<StoreRepository>();

        let store_row: StoreRow = store_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get store {}", id));

        Store { store_row }
    }

    pub async fn transact(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the transact")] id: String,
    ) -> Transact {
        let transact_repository = ctx.get_repository::<TransactRepository>();

        let transact_row: TransactRow = transact_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get transact {}", id));

        Transact { transact_row }
    }

    pub async fn transact_line(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the transact line")] id: String,
    ) -> TransactLine {
        let transact_line_repository = ctx.get_repository::<TransactLineRepository>();

        let transact_line_row: TransactLineRow = transact_line_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get transact line {}", id));

        TransactLine { transact_line_row }
    }

    pub async fn requisition(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the requisition")] id: String,
    ) -> Requisition {
        let requisition_repository = ctx.get_repository::<RequisitionRepository>();

        let requisition_row: RequisitionRow = requisition_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get requisition {}", id));

        Requisition { requisition_row }
    }

    pub async fn item(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the item")] id: String,
    ) -> Item {
        let item_repository = ctx.get_repository::<ItemRepository>();

        let item_row: ItemRow = item_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get item {}", id));

        Item { item_row }
    }

    pub async fn items(&self, ctx: &Context<'_>) -> Vec<Item> {
        let item_repository = ctx.get_repository::<ItemRepository>();

        let item_rows: Vec<ItemRow> = item_repository
            .find_all()
            .await
            .unwrap_or_else(|_| panic!("Failed to get items"));

        item_rows
            .into_iter()
            .map(|item_row| Item { item_row })
            .collect()
    }

    pub async fn item_line(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the item line")] id: String,
    ) -> ItemLine {
        let item_line_repository = ctx.get_repository::<ItemLineRepository>();

        let item_line_row: ItemLineRow = item_line_repository
            .find_one_by_id(&id)
            .await
            .unwrap_or_else(|_| panic!("Failed to get item line {}", id));

        ItemLine { item_line_row }
    }
}
