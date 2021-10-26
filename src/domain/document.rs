use std::convert::TryFrom;

use chrono::{DateTime, Utc};

use crate::{
    database::schema::DocumentRow, domain::document_canonical_json::CanonicalJsonValue,
    util::auth::sha256,
};
use serde::{self, Deserialize, Serialize};

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

/// Like Document but without id
#[derive(Debug, Serialize, Deserialize)]
pub struct RawDocument {
    pub name: String,
    pub parents: Vec<String>,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: serde_json::Value,
}

fn document_id(raw: &RawDocument) -> Result<String, String> {
    let value = serde_json::to_value(raw).map_err(|err| format!("{:?}", err))?;
    let canonical_value = CanonicalJsonValue::from(value);
    let str = canonical_value.to_string();
    Ok(sha256(&str))
}
fn finalise_document(raw: RawDocument) -> Result<Document, String> {
    let id = document_id(&raw)?;
    let RawDocument {
        name,
        parents,
        author,
        timestamp,
        type_,
        data,
    } = raw;
    Ok(Document {
        id,
        name,
        parents,
        author,
        timestamp,
        type_,
        data,
    })
}

struct DocumentService {}

impl DocumentService {}

#[cfg(test)]
mod document_id_test {
    use chrono::TimeZone;
    use serde_json::*;

    use super::*;

    #[test]
    fn test_id() {
        let raw = RawDocument {
            name: "name".to_string(),
            parents: vec!["p1".to_string()],
            author: "author".to_string(),
            timestamp: Utc.timestamp_millis(1000),
            type_: "test".to_string(),
            data: json!({
              "b": 0.3453333,
              "a": "avalue",
            }),
        };
        let document = finalise_document(raw).unwrap();
        let expected_json_string = r#"{"author":"author","data":{"a":"avalue","b":0.3453333},"name":"name","parents":["p1"],"timestamp":"1970-01-01T00:00:01Z","type":"test"}"#;
        let expected_id = sha256(expected_json_string);
        assert_eq!(document.id, expected_id);
    }
}
