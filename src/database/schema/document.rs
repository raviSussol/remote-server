use super::diesel_schema::document;

use chrono::NaiveDateTime;

#[derive(Clone, Queryable, Insertable, AsChangeset, Debug, PartialEq)]
#[table_name = "document"]
pub struct DocumentRow {
    /// The document data hash
    pub id: String,
    /// Document path and name
    pub name: String,
    /// Stringified array of parents
    pub parents: String,
    /// Id of the author who edited this document version
    pub author: String,
    /// The timestamp of this document version
    pub timestamp: NaiveDateTime,
    /// Type of the containing data
    pub type_: String,
    /// The actual document data
    pub data: String,
}
