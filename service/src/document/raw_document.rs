use chrono::{DateTime, Utc};
use domain::{document::Document, json_schema::JSONSchema};
use serde::{Deserialize, Serialize};
use util::{canonical_json::CanonicalJsonValue, hash::sha256};

/// Like Document but without id
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawDocument {
    pub name: String,
    pub parents: Vec<String>,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<JSONSchema>,
}

impl RawDocument {
    /// Calculates the document id
    pub fn document_id(&self) -> Result<String, String> {
        let value = serde_json::to_value(self).map_err(|err| format!("{:?}", err))?;
        let canonical_value = CanonicalJsonValue::from(value);
        let str = canonical_value.to_string();
        Ok(sha256(&str))
    }

    /// RawDocument can't be used afterwards (self parameter ensures that)
    pub fn finalise(self) -> Result<Document, String> {
        let id = self.document_id()?;
        let RawDocument {
            name,
            parents,
            author,
            timestamp,
            type_,
            data,
            schema,
        } = self;
        Ok(Document {
            id,
            name,
            parents,
            author,
            timestamp,
            type_,
            data,
            schema,
        })
    }
}

#[cfg(test)]
mod document_id_test {
    use chrono::TimeZone;
    use serde_json::*;

    use super::*;

    #[test]
    fn test_document_id() {
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
            schema: None,
        };
        let document = raw.finalise().unwrap();
        let expected_json_string = r#"{"author":"author","data":{"a":"avalue","b":0.3453333},"name":"name","parents":["p1"],"timestamp":"1970-01-01T00:00:01Z","type":"test"}"#;
        let expected_id = sha256(expected_json_string);
        assert_eq!(document.id, expected_id);
    }
}
