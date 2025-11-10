use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove symbolID fields that contain only localID and/or sessionID.
///
/// This transformation removes `symbolID` objects when they contain only
/// the fields `localID` and/or `sessionID`. If the `symbolID` object has
/// any additional fields beyond these two, it is preserved.
///
/// # Examples
///
/// Removed (only standard fields):
/// ```json
/// {
///   "symbolID": {
///     "localID": 10596,
///     "sessionID": 4331
///   }
/// }
/// ```
///
/// Preserved (has extra field):
/// ```json
/// {
///   "symbolID": {
///     "localID": 10596,
///     "sessionID": 4331,
///     "customField": "value"
///   }
/// }
/// ```
pub fn remove_symbol_id_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check if this object has a "symbolID" field that should be removed
            if let Some(symbol_id_value) = map.get("symbolID") {
                if should_remove_symbol_id(symbol_id_value) {
                    map.remove("symbolID");
                }
            }

            // Recurse into all remaining values
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val)?;
                }
            }
        }
        JsonValue::Array(arr) => {
            // Recurse into array elements
            for val in arr.iter_mut() {
                transform_recursive(val)?;
            }
        }
        _ => {
            // Primitives - nothing to do
        }
    }

    Ok(())
}

/// Determine if a symbolID field should be removed.
///
/// Returns true if the value is an object containing only the fields
/// "localID" and/or "sessionID" with no other fields.
fn should_remove_symbol_id(value: &JsonValue) -> bool {
    if let JsonValue::Object(map) = value {
        // Check if all keys are in the allowed set
        for key in map.keys() {
            if key != "localID" && key != "sessionID" {
                // Found a field that's not localID or sessionID
                return false;
            }
        }
        // All keys are either localID or sessionID (or empty)
        true
    } else {
        // Not an object, don't remove
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_symbol_id_with_both_fields() {
        let mut input = json!({
            "name": "Navigation",
            "symbolID": {
                "localID": 10596,
                "sessionID": 4331
            },
            "size": {
                "x": 375.0,
                "y": 122.0
            }
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "name": "Navigation",
            "size": {
                "x": 375.0,
                "y": 122.0
            }
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_remove_symbol_id_with_only_local_id() {
        let mut input = json!({
            "name": "Test",
            "symbolID": {
                "localID": 123
            }
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "name": "Test"
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_remove_symbol_id_with_only_session_id() {
        let mut input = json!({
            "name": "Test",
            "symbolID": {
                "sessionID": 456
            }
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "name": "Test"
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_keep_symbol_id_with_extra_fields() {
        let mut input = json!({
            "name": "Test",
            "symbolID": {
                "localID": 123,
                "sessionID": 456,
                "customField": "value"
            }
        });

        let expected = input.clone();
        remove_symbol_id_fields(&mut input).unwrap();

        assert_eq!(input, expected);
    }

    #[test]
    fn test_keep_symbol_id_with_different_field() {
        let mut input = json!({
            "name": "Test",
            "symbolID": {
                "localID": 123,
                "customField": "value"
            }
        });

        let expected = input.clone();
        remove_symbol_id_fields(&mut input).unwrap();

        assert_eq!(input, expected);
    }

    #[test]
    fn test_remove_empty_symbol_id() {
        let mut input = json!({
            "name": "Test",
            "symbolID": {}
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "name": "Test"
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_nested_symbol_id_removal() {
        let mut input = json!({
            "children": [
                {
                    "name": "Child1",
                    "symbolID": {
                        "localID": 1,
                        "sessionID": 2
                    }
                },
                {
                    "name": "Child2",
                    "nested": {
                        "symbolID": {
                            "localID": 3,
                            "sessionID": 4
                        }
                    }
                }
            ]
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "children": [
                {
                    "name": "Child1"
                },
                {
                    "name": "Child2",
                    "nested": {}
                }
            ]
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_mixed_symbol_ids() {
        let mut input = json!({
            "nodes": [
                {
                    "name": "Remove",
                    "symbolID": {
                        "localID": 1,
                        "sessionID": 2
                    }
                },
                {
                    "name": "Keep",
                    "symbolID": {
                        "localID": 3,
                        "sessionID": 4,
                        "extra": "field"
                    }
                }
            ]
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "nodes": [
                {
                    "name": "Remove"
                },
                {
                    "name": "Keep",
                    "symbolID": {
                        "localID": 3,
                        "sessionID": 4,
                        "extra": "field"
                    }
                }
            ]
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_symbol_id_not_object() {
        // If symbolID is not an object, keep it (shouldn't happen, but be defensive)
        let mut input = json!({
            "name": "Test",
            "symbolID": "string_value"
        });

        let expected = input.clone();
        remove_symbol_id_fields(&mut input).unwrap();

        assert_eq!(input, expected);
    }

    #[test]
    fn test_deeply_nested_structure() {
        let mut input = json!({
            "level1": {
                "symbolID": {
                    "localID": 1
                },
                "level2": {
                    "symbolID": {
                        "sessionID": 2
                    },
                    "level3": {
                        "symbolID": {
                            "localID": 3,
                            "sessionID": 4
                        }
                    }
                }
            }
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "level1": {
                "level2": {
                    "level3": {}
                }
            }
        });

        assert_eq!(input, expected);
    }
}
