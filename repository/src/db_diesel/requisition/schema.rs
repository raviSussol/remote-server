use chrono::NaiveDateTime;
use diesel_derive_enum::DbEnum;
use domain::{DatetimeFilter, EqualFilter, SimpleStringFilter, Sort};

// Requisition Row

#[derive(DbEnum, Debug, Clone, PartialEq, Eq)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum RequisitionRowType {
    Request,
    Response,
}
#[derive(DbEnum, Debug, Clone, PartialEq, Eq)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum RequisitionRowStatus {
    Draft,
    New,
    Sent,
    Finalised,
}

#[derive(Clone, Queryable, Insertable, AsChangeset, Debug, PartialEq)]
#[table_name = "requisition"]
pub struct RequisitionRow {
    pub id: String,
    pub requisition_number: i64,
    pub name_id: String,
    pub store_id: String,
    #[column_name = "type_"]
    pub r#type: RequisitionRowType,
    pub status: RequisitionRowStatus,
    pub created_datetime: NaiveDateTime,
    pub sent_datetime: Option<NaiveDateTime>,
    pub finalised_datetime: Option<NaiveDateTime>,
    pub colour: Option<String>,
    pub comment: Option<String>,
    pub their_reference: Option<String>,
    pub max_months_of_stock: f64,
    pub threshold_months_of_stock: f64,
    pub linked_requisition_id: Option<String>,
}

// Diesel

table! {
    requisition (id) {
        id -> Text,
        requisition_number -> Bigint,
        name_id -> Text,
        store_id -> Text,
        #[sql_name = "type"] type_ -> crate::RequisitionRowTypeMapping, // super does not work
        #[sql_name = "status"] status -> crate::RequisitionRowStatusMapping, // super does not work
        created_datetime -> Timestamp,
        sent_datetime -> Nullable<Timestamp>,
        finalised_datetime -> Nullable<Timestamp>,
        colour -> Nullable<Text>,
        comment -> Nullable<Text>,
        their_reference -> Nullable<Text>,
        max_months_of_stock -> Double,
        threshold_months_of_stock -> Double,
        linked_requisition_id -> Nullable<Text>,
    }
}

pub use requisition as requisition_schema;
pub use requisition::dsl as requisition_dsl;
pub use requisition::table as requisition_table;

// Requsition Row Filter and Sort

#[derive(Clone, Debug, PartialEq)]
pub struct RequisitionFilter {
    pub id: Option<EqualFilter<String>>,
    pub requisition_number: Option<EqualFilter<i64>>,
    pub r#type: Option<EqualFilter<RequisitionRowType>>,
    pub status: Option<EqualFilter<RequisitionRowStatus>>,
    pub created_datetime: Option<DatetimeFilter>,
    pub sent_datetime: Option<DatetimeFilter>,
    pub finalised_datetime: Option<DatetimeFilter>,
    pub name_id: Option<EqualFilter<String>>,
    pub name: Option<SimpleStringFilter>,
    pub colour: Option<EqualFilter<String>>,
    pub their_reference: Option<SimpleStringFilter>,
    pub comment: Option<SimpleStringFilter>,
    pub store_id: Option<EqualFilter<String>>,
    pub linked_requisition_id: Option<EqualFilter<String>>,
}

#[derive(PartialEq, Debug)]
pub enum RequisitionSortField {
    RequisitionNumber,
    Type,
    Status,
    OtherPartyName,
    SentDatetime,
    CreatedDatetime,
    FinalisedDatetime,
}

pub type RequisitionSort = Sort<RequisitionSortField>;

impl RequisitionFilter {
    pub fn new() -> RequisitionFilter {
        RequisitionFilter {
            id: None,
            requisition_number: None,
            r#type: None,
            status: None,
            created_datetime: None,
            sent_datetime: None,
            finalised_datetime: None,
            name_id: None,
            name: None,
            colour: None,
            their_reference: None,
            comment: None,
            store_id: None,
            linked_requisition_id: None,
        }
    }

    pub fn id(mut self, filter: EqualFilter<String>) -> Self {
        self.id = Some(filter);
        self
    }

    pub fn name(mut self, filter: SimpleStringFilter) -> Self {
        self.name = Some(filter);
        self
    }

    pub fn status(mut self, filter: EqualFilter<RequisitionRowStatus>) -> Self {
        self.status = Some(filter);
        self
    }

    pub fn comment(mut self, filter: SimpleStringFilter) -> Self {
        self.comment = Some(filter);
        self
    }

    pub fn requisition_number(mut self, filter: EqualFilter<i64>) -> Self {
        self.requisition_number = Some(filter);
        self
    }

    pub fn store_id(mut self, filter: EqualFilter<String>) -> Self {
        self.store_id = Some(filter);
        self
    }

    pub fn r#type(mut self, filter: EqualFilter<RequisitionRowType>) -> Self {
        self.r#type = Some(filter);
        self
    }

    pub fn linked_requisition_id(mut self, filter: EqualFilter<String>) -> Self {
        self.linked_requisition_id = Some(filter);
        self
    }
}

impl RequisitionRowStatus {
    pub fn equal_to(&self) -> EqualFilter<RequisitionRowStatus> {
        EqualFilter {
            equal_to: Some(self.clone()),
            not_equal_to: None,
            equal_any: None,
            not_equal_all: None,
        }
    }
}

impl RequisitionRowType {
    pub fn equal_to(&self) -> EqualFilter<RequisitionRowType> {
        EqualFilter {
            equal_to: Some(self.clone()),
            not_equal_to: None,
            equal_any: None,
            not_equal_all: None,
        }
    }
}
