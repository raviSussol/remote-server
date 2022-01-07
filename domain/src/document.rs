use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug)]
pub struct Document {
    /// The document data hash
    pub id: String,
    /// Document path and name
    pub name: String,
    /// Document parents
    pub parents: Vec<String>,
    /// Id of the author who edited this document version
    pub author: String,
    /// The timestamp of this document version
    pub timestamp: DateTime<Utc>,
    /// Type of the containing data
    pub type_: String,
    /// The actual document data
    pub data: serde_json::Value,
}

#[derive(Clone)]
pub struct AncestorDetail {
    pub id: String,
    pub parents: Vec<String>,
    pub timestamp: NaiveDateTime,
}
