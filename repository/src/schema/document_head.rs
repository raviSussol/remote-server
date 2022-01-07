use super::diesel_schema::document_head;

/// Hold the a reference to the latest document version
#[derive(Clone, Queryable, Insertable, AsChangeset, Debug, PartialEq)]
#[table_name = "document_head"]
pub struct DocumentHeadRow {
    /// Row id in the format "{name}@{store}"
    pub id: String,
    /// The store this head refers too. This mean we can keep track of heads from multiple stores
    /// and merge them when needed.
    pub store: String,
    /// The document name
    pub name: String,
    /// The current document version (hash)
    pub head: String,
}
