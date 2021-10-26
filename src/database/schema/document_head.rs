use super::diesel_schema::document_head;

/// Hold the a reference to the latest document version
#[derive(Clone, Queryable, Insertable, AsChangeset, Debug, PartialEq)]
#[table_name = "document_head"]
pub struct DocumentHeadRow {
    /// The document name
    pub id: String,
    /// The current document version (hash)
    pub document_id: String,
}
