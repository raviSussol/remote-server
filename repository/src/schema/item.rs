use super::diesel_schema::item;
use diesel_derive_enum::DbEnum;

#[derive(DbEnum, Debug, Clone, PartialEq, Eq)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum ItemRowType {
    Stock,
    Service,
    NonStock,
}

#[derive(Clone, Insertable, Queryable, Debug, PartialEq, Eq, AsChangeset)]
#[table_name = "item"]
pub struct ItemRow {
    pub id: String,
    pub name: String,
    pub code: String,
    pub unit_id: Option<String>,
    #[column_name = "type_"]
    pub r#type: ItemRowType,
    pub universal_code: String,
}
