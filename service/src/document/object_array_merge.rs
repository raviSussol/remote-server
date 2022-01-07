use serde_json::Value;

use super::merge::{three_way_merge_object, two_way_merge_object, ConflictSolver, MergeObject};

// Todo make this configurable?
const OBJECT_ARRAY_KEY: &str = "key";

/// Merges arrays that only contain objects and all objects contain a string "key".
/// For example:
/// ```json
/// [
///   {
///     "key": "key1",
///     "value": ...
///   },
///   {
///     "key": "key2",
///     "value:" ...
///   }
/// ]
/// ```
/// If the array contains elements that don't fit this pattern the parent solver is used to solve
/// the conflict.
pub struct ObjectArrayConflictSolver {
    parent: Box<dyn ConflictSolver>,
}

impl ConflictSolver for ObjectArrayConflictSolver {
    fn solve(&self, our: &Value, their: &Value, base: Option<&Value>) -> Value {
        if let Some(base) = base {
            // three way merge
            match (our, their, base) {
                (Value::Array(o), Value::Array(t), Value::Array(b)) => {
                    if let Some(merged) = three_way_merge_object_array(
                        o,
                        t,
                        b,
                        self.parent.as_ref(),
                        OBJECT_ARRAY_KEY,
                    ) {
                        return Value::Array(merged);
                    }
                }
                _ => {}
            }
        } else {
            // two way merge
            match (our, their) {
                (Value::Array(o), Value::Array(t)) => {
                    if let Some(merged) =
                        two_way_merge_object_array(o, t, self.parent.as_ref(), OBJECT_ARRAY_KEY)
                    {
                        return Value::Array(merged);
                    }
                }
                _ => {}
            };
        }
        return self.parent.solve(our, their, None);
    }
}

/// Test that all members are objects and that all object contain a `key` of type string.
/// If this is the case the array members are put into an object with the key value as object key.
fn object_array_to_object(array: &Vec<Value>, key: &str) -> Option<MergeObject> {
    let mut result = MergeObject::new();
    for item in array {
        let obj = match item {
            Value::Object(obj) => obj,
            _ => return None,
        };

        let value = match obj.get(key) {
            Some(value) => value,
            None => return None,
        };

        let key_value = match value {
            Value::String(key) => key,
            _ => return None,
        };

        result.insert(key_value.to_string(), item.clone());
    }
    Some(result)
}

fn two_way_merge_object_array(
    ours: &Vec<serde_json::Value>,
    theirs: &Vec<serde_json::Value>,
    strategy: &dyn ConflictSolver,
    array_object_key: &str,
) -> Option<Vec<serde_json::Value>> {
    let (ours, theirs) = match (
        object_array_to_object(ours, array_object_key),
        object_array_to_object(theirs, array_object_key),
    ) {
        (Some(ours), Some(theirs)) => (ours, theirs),
        _ => return None,
    };

    let merged = two_way_merge_object(&ours, &theirs, strategy);
    let mut merged_array: Vec<Value> = merged.into_iter().map(|(_, value)| value).collect();
    merged_array.sort_by(|a, b| {
        let str_a = format!("{}", a[array_object_key]);
        let str_b = format!("{}", b[array_object_key]);
        str_a.cmp(&str_b)
    });
    Some(merged_array)
}

fn three_way_merge_object_array(
    ours: &Vec<serde_json::Value>,
    theirs: &Vec<serde_json::Value>,
    base: &Vec<serde_json::Value>,
    strategy: &dyn ConflictSolver,
    array_object_key: &str,
) -> Option<Vec<serde_json::Value>> {
    let (ours, theirs, base) = match (
        object_array_to_object(ours, array_object_key),
        object_array_to_object(theirs, array_object_key),
        object_array_to_object(base, array_object_key),
    ) {
        (Some(ours), Some(theirs), Some(base)) => (ours, theirs, base),
        _ => return None,
    };

    let merged = three_way_merge_object(&ours, &theirs, &base, strategy);
    let mut merged_array: Vec<Value> = merged.into_iter().map(|(_, value)| value).collect();
    merged_array.sort_by(|a, b| {
        let str_a = format!("{}", a[array_object_key]);
        let str_b = format!("{}", b[array_object_key]);
        str_a.cmp(&str_b)
    });
    Some(merged_array)
}

#[cfg(test)]
mod object_array_merge_test {
    use assert_json_diff::assert_json_eq;
    use serde_json::*;

    use crate::document::merge::{three_way_merge, two_way_merge, TakeOurConflictSolver};

    use super::ObjectArrayConflictSolver;

    #[test]
    fn test_object_array_two_way_merge() {
        let theirs = json!({
          "array1": [{
            "key": "1",
            "value1": "value1",
          },
          {
            "key": "2",
            "value1": "value1",
          },
          {
            "key": "3",
            "value1": "value1",
          }]
        });
        let ours = json!({
          "array1": [{
            "key": "1",
            "value1": "value1",
          },
          {
            "key": "2",
            "value1": "value2",
          }]
        });
        let solver = ObjectArrayConflictSolver {
            parent: Box::new(TakeOurConflictSolver {}),
        };
        let result = two_way_merge(&ours, &theirs, &solver);
        assert_json_eq!(
            &result,
            json!({
              "array1": [{
                "key": "1",
                "value1": "value1",
              },
              {
                "key": "2",
                "value1": "value2",
              },
              {
                "key": "3",
                "value1": "value1",
              }]
            })
        );
    }

    #[test]
    fn test_object_array_three_way_merge() {
        let base = json!({
          "array1": [{
            "key": "1",
            "value1": "value1",
          },
          {
            "key": "2",
            "value1": "value1",
          }]
        });
        let theirs = json!({
          "array1": [{
            "key": "2",
            "value1": "theirs",
          },
          {
            "key": "3",
            "value1": "theirs",
          }]
        });
        let ours = json!({
          "array1": [{
            "key": "1",
            "value1": "value1",
          },
          {
            "key": "2",
            "value1": "ours",
          }]
        });
        let solver = ObjectArrayConflictSolver {
            parent: Box::new(TakeOurConflictSolver {}),
        };
        let result = three_way_merge(&ours, &theirs, &base, &solver);
        assert_json_eq!(
            &result,
            json!({
              "array1": [{
                "key": "2",
                "value1": "ours",
              },
              {
                "key": "3",
                "value1": "theirs",
              }]
            })
        );
    }
}
