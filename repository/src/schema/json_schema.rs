use super::diesel_schema::json_schema;

#[derive(Clone, Queryable, Insertable, AsChangeset, Debug, PartialEq)]
#[table_name = "json_schema"]
pub struct JSONSchemaRow {
    /// The json schema id
    pub id: String,
    /// Document path and name
    pub schema: String,
}
