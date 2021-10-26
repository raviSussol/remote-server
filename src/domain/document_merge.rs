use chrono::{DateTime, Utc};
use serde_json::Value;

type MergeObject = serde_json::Map<String, serde_json::Value>;

pub trait ConflictSolver {
    fn solve(&self, our: &Value, their: &Value, base: Option<&Value>) -> Value;
}

pub struct TakeOurConflictSolver {}

impl ConflictSolver for TakeOurConflictSolver {
    fn solve(&self, our: &Value, _: &Value, _: Option<&Value>) -> Value {
        our.clone()
    }
}

pub struct TakeLatestConflictSolver {
    our: DateTime<Utc>,
    their: DateTime<Utc>,
}

impl TakeLatestConflictSolver {
    pub fn new(our: DateTime<Utc>, their: DateTime<Utc>) -> Self {
        TakeLatestConflictSolver { our, their }
    }
}

impl ConflictSolver for TakeLatestConflictSolver {
    fn solve(&self, our: &Value, their: &Value, _: Option<&Value>) -> Value {
        if self.our > self.their {
            our.clone()
        } else {
            their.clone()
        }
    }
}

pub fn two_way_merge(ours: &Value, theirs: &Value, strategy: &dyn ConflictSolver) -> Value {
    match (ours, theirs) {
        (Value::Object(o), Value::Object(t)) => {
            return two_way_merge_object(o, t, strategy);
        }
        (o, t) => {
            let merge = two_way_merge_value(Some(o), Some(t), strategy);
            merge.unwrap_or(Value::Null)
        }
    }
}

fn two_way_merge_object(
    ours: &MergeObject,
    theirs: &MergeObject,
    strategy: &dyn ConflictSolver,
) -> Value {
    let all_keys: Vec<String> = ours
        .keys()
        .into_iter()
        .chain(theirs.keys().into_iter())
        .map(|it| it.to_owned())
        .collect();

    let mut result = MergeObject::new();
    for key in all_keys {
        let merged = two_way_merge_value(ours.get(&key), theirs.get(&key), strategy);
        merged.map(|v| result.insert(key, v));
    }

    Value::Object(result)
}

fn two_way_merge_value(
    ours: Option<&Value>,
    theirs: Option<&Value>,
    strategy: &dyn ConflictSolver,
) -> Option<Value> {
    match (ours, theirs) {
        (Some(our), Some(their)) => match (our, their) {
            (Value::Object(o), Value::Object(t)) => Some(two_way_merge_object(o, t, strategy)),
            _ => Some(strategy.solve(our, their, None)),
        },
        (Some(our), None) => Some(our.clone()),
        (None, Some(their)) => Some(their.clone()),
        (None, None) => None,
    }
}

/// Merge theirs into ours using `base` as a common ancestor
pub fn three_way_merge(
    ours: &Value,
    theirs: &Value,
    base: &Value,
    strategy: &dyn ConflictSolver,
) -> Value {
    match (ours, theirs, base) {
        (Value::Object(o), Value::Object(t), Value::Object(b)) => {
            return three_way_merge_object(o, t, b, strategy);
        }
        (o, t, b) => {
            let merge = three_way_merge_value(Some(o), Some(t), Some(b), strategy);
            merge.unwrap_or(Value::Null)
        }
    }
}

fn three_way_merge_object(
    ours: &MergeObject,
    theirs: &MergeObject,
    base: &MergeObject,
    strategy: &dyn ConflictSolver,
) -> Value {
    let all_keys: Vec<String> = ours
        .keys()
        .into_iter()
        .chain(theirs.keys().into_iter())
        .map(|it| it.to_owned())
        .collect();

    let mut result = MergeObject::new();
    for key in all_keys {
        let merged =
            three_way_merge_value(ours.get(&key), theirs.get(&key), base.get(&key), strategy);
        merged.map(|v| result.insert(key, v));
    }

    Value::Object(result)
}

fn three_way_merge_value(
    ours: Option<&Value>,
    theirs: Option<&Value>,
    base: Option<&Value>,
    strategy: &dyn ConflictSolver,
) -> Option<Value> {
    match (ours, theirs, base) {
        (Some(our), Some(their), Some(base)) => match (our, their, base) {
            (Value::Object(o), Value::Object(t), Value::Object(b)) => {
                Some(three_way_merge_object(o, t, b, strategy))
            }
            _ => {
                if our == their {
                    Some(our.clone())
                } else if our == base {
                    Some(their.clone())
                } else if their == base {
                    Some(our.clone())
                } else {
                    Some(strategy.solve(our, their, Some(base)))
                }
            }
        },
        (Some(our), Some(their), None) => {
            let merged = two_way_merge(our, their, strategy);
            Some(merged)
        }
        (Some(_), None, Some(_)) => {
            // removed in theirs
            None
        }
        (None, Some(_), Some(_)) => {
            // removed in ours
            None
        }
        (Some(our), None, None) => {
            // added in our's
            Some(our.clone())
        }
        (None, Some(their), None) => {
            // Added in their's
            Some(their.clone())
        }
        (None, None, Some(_)) => {
            // nothing to do
            None
        }
        (None, None, None) => {
            // should not happen
            None
        }
    }
}

#[cfg(test)]
mod two_way_merge_test {
    use assert_json_diff::assert_json_eq;
    use serde_json::*;

    use crate::domain::document_merge::{two_way_merge, TakeOurConflictSolver};

    #[test]
    fn test_simple_merge() {
        let theirs = json!({
          "value1": "string2",
          "value2": true,
          "value4": 50,
          "value5": 5
        });
        let ours = json!({
          "value1": "string",
          "value2": false,
          "value3": 30,
          "value4": 50
        });
        let result = two_way_merge(&ours, &theirs, &TakeOurConflictSolver {});
        assert_json_eq!(
            &result,
            json!({
              "value1": "string",
              "value2": false,
              "value3": 30,
              "value4": 50,
              "value5": 5
            })
        );
    }

    #[test]
    fn test_simple_nested() {
        let theirs = json!({
          "value1": "string",
          "value2": true,
          "value3": {
            "obj": {
              "value1": 1,
            },
            "value1": "v1",
            "value2": 2
          }
        });
        let ours = json!({
          "value1": "string",
          "value2": true,
          "value3": {
            "obj": {
              "value1": 2,
              "value2": 22,
            },
            "value1": "v1"
          }
        });
        let result = two_way_merge(&ours, &theirs, &TakeOurConflictSolver {});
        assert_json_eq!(
            &result,
            json!({
              "value1": "string",
              "value2": true,
              "value3": {
                "obj": {
                  "value1": 2,
                  "value2": 22,
                },
                "value1": "v1",
                "value2": 2
              }
            })
        );
    }
}

#[cfg(test)]
mod three_way_merge_test {
    use assert_json_diff::assert_json_eq;
    use serde_json::*;

    use crate::domain::document_merge::{three_way_merge, TakeOurConflictSolver};

    #[test]
    fn test_simple_merge() {
        let base = json!({
          "value1": "string",
          "value2": true,
          "value3": 30,
          "value4": 50
        });
        let theirs = json!({
          "value1": "string2",
          "value2": true,
          "value4": 50
        });
        let ours = json!({
          "value1": "string",
          "value2": false,
          "value3": 30,
          "value4": 50
        });
        let result = three_way_merge(&ours, &theirs, &base, &TakeOurConflictSolver {});
        assert_json_eq!(
            &result,
            json!({
              "value1": "string2",
              "value2": false,
              "value4": 50
            })
        );
    }

    #[test]
    fn test_simple_merge_conflict() {
        // test conflict
        let base = json!({
          "value1": "string",
          "value2": true,
          "value3": 30,
          "value4": 50
        });
        let theirs = json!({
          "value1": "string2",
          "value2": false,
          "value4": 51
        });
        let ours = json!({
          "value1": "string3",
          "value2": false,
          "value3": 30,
          "value4": 52
        });
        let result = three_way_merge(&ours, &theirs, &base, &TakeOurConflictSolver {});
        assert_json_eq!(
            &result,
            json!({
              "value1": "string3",
              "value2": false,
              "value4": 52
            })
        );
    }

    #[test]
    fn test_simple_merge_nested() {
        // test conflict
        let base = json!({
          "value1": "string",
          "obj": {
            "i1": 1,
            "obj2": {
              "str": "str",
            }
          }
        });
        let theirs = json!({
          "value1": "string",
          "obj": {
            "i1": 2,
            "obj2": {
              "str": "str",
            }
          }
        });
        let ours = json!({
          "value1": "string2",
          "obj": {
            "i1": 1,
            "obj2": {
              "str": "str2",
            }
          }
        });
        let result = three_way_merge(&ours, &theirs, &base, &TakeOurConflictSolver {});
        assert_json_eq!(
            &result,
            json!({
              "value1": "string2",
              "obj": {
                "i1": 2,
                "obj2": {
                  "str": "str2",
                }
              }
            })
        );

        // remove whole nested obj
        let ours = json!({
          "value1": "string2",
          "obj": {
            "i1": 1,
          }
        });
        let result = three_way_merge(&ours, &theirs, &base, &TakeOurConflictSolver {});
        assert_json_eq!(
            &result,
            json!({
              "value1": "string2",
              "obj": {
                "i1": 2,
              }
            })
        );
    }

    #[test]
    fn test_simple_merge_different_shapes() {
        // test conflict
        let base = json!({
          "value1": "string",
          "value2": true,
          "value3": 30,
          "obj": {
            "v": 1,
          }
        });
        let theirs = json!({
          "value1": 1,
          "value2": "str",
          "value3": 30,
          "obj": {
            "v": 2
          }
        });
        let ours = json!({
          "value1": "string",
          "value2": true,
          "value3": false,
          "obj": 10,
        });
        let result = three_way_merge(&ours, &theirs, &base, &TakeOurConflictSolver {});
        assert_json_eq!(
            &result,
            json!({
              "value1": 1,
              "value2": "str",
              "value3": false,
              "obj": 10,
            })
        );
    }
}
