use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct GraphQlQuery {
    pub query: String,
    /// Must be an Object. If `dataId` is set it will be overwritten.
    pub variables: Option<Value>,
}

impl GraphQlQuery {
    /// Create query variables for the query
    pub fn query_variables(&self, store_id: &str, data_id: &str) -> Value {
        let mut variables = match &self.variables {
            Some(variables) => {
                if matches!(variables, Value::Object(_)) {
                    variables.clone()
                } else {
                    // ensure variables are an object
                    serde_json::json!({})
                }
            }
            None => serde_json::json!({}),
        };
        variables["storeId"] = Value::String(store_id.to_string());
        variables["dataId"] = Value::String(data_id.to_string());
        variables
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DefaultQuery {
    Invoice,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ReportRef {
    /// The id of the source report definition that is referred to by this reference
    pub source: String,
    /// The name of the entry in the referred report definition (only needed if different to local
    /// name)
    pub source_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TeraTemplate {
    pub output: ReportOutputType,
    pub template: String,
}

/// The output format that is produced by a report
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ReportOutputType {
    Html,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ReportDefinitionEntry {
    TeraTemplate(TeraTemplate),
    /// Custom http query
    GraphGLQuery(GraphQlQuery),
    /// Use default predefined query
    DefaultQuery(DefaultQuery),
    Resource(serde_json::Value),
    /// Entry reference to another report definition
    Ref(ReportRef),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ReportDefinition {
    pub entries: HashMap<String, ReportDefinitionEntry>,
}

#[cfg(test)]
mod report_dsl_test {
    use std::collections::HashMap;

    use serde_json::json;

    use crate::report::definition::{
        DefaultQuery, ReportDefinition, ReportDefinitionEntry, ReportOutputType, ReportRef,
        TeraTemplate,
    };

    #[test]
    fn parse_template() {
        let template_data = r#"Hello World (Shipment id: {{id}})
        Some query data: {{data.value}}
        Some resource data: {{res.icon1}} and {{res.mainIcon}},
        "#;
        let template = json!({
          "entries": {
              "template": {
                  "type": "TeraTemplate",
                  "data": {
                      "output": "Html",
                      "template": template_data,
                  }
              },
              "local_footer.html": {
                  "type": "Ref",
                  "data": {
                      "source": "other_report_def",
                      "source_name": "footer.html",
                  }
              },
              "query": {
                  "type": "DefaultQuery",
                  "data": "OutboundShipment"
              },
              "icon": {
                  "type": "Resource",
                  "data": "IconData"
              },
              "mainIcon": {
                  "type": "Ref",
                  "data": {
                      "source": "other_report_def",
                  }
              }
          }
        });
        let report: ReportDefinition = serde_json::from_value(template).unwrap();
        assert_eq!(
            report,
            ReportDefinition {
                entries: HashMap::from([
                    (
                        "local_footer.html".to_string(),
                        ReportDefinitionEntry::Ref(ReportRef {
                            source: "other_report_def".to_string(),
                            source_name: Some("footer.html".to_string()),
                        })
                    ),
                    (
                        "template".to_string(),
                        ReportDefinitionEntry::TeraTemplate(TeraTemplate {
                            output: ReportOutputType::Html,
                            template: template_data.to_string()
                        })
                    ),
                    (
                        "query".to_string(),
                        ReportDefinitionEntry::DefaultQuery(DefaultQuery::Invoice)
                    ),
                    (
                        "icon".to_string(),
                        ReportDefinitionEntry::Resource(json!("IconData"))
                    ),
                    (
                        "mainIcon".to_string(),
                        ReportDefinitionEntry::Ref(ReportRef {
                            source: "other_report_def".to_string(),
                            source_name: None
                        })
                    )
                ]),
            }
        )
    }
}
