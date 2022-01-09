use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JSONSchema {
    pub id: String,
    pub schema: serde_json::Value,
}
