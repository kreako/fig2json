use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove phase fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes all "phase" fields.
/// These fields contain Figma internal state (typically {"__enum__": "NodePhase", "value": "CREATED"})
/// that are not needed for HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all phase fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_phase_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "phase": {
///         "__enum__": "NodePhase",
///         "value": "CREATED"
///     },
///     "visible": true
/// });
/// remove_phase_fields(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_phase_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove phase fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the "phase" field if it exists
            map.remove("phase");

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_phase_simple() {
        let mut tree = json!({
            "name": "Rectangle",
            "phase": {
                "__enum__": "NodePhase",
                "value": "CREATED"
            },
            "visible": true
        });

        remove_phase_fields(&mut tree).unwrap();

        assert!(tree.get("phase").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_phase_nested() {
        let mut tree = json!({
            "name": "Root",
            "phase": {
                "__enum__": "NodePhase",
                "value": "CREATED"
            },
            "children": [
                {
                    "name": "Child1",
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "CREATED"
                    }
                },
                {
                    "name": "Child2",
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "DELETED"
                    }
                }
            ]
        });

        remove_phase_fields(&mut tree).unwrap();

        // Root phase should be removed
        assert!(tree.get("phase").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Root"));

        // Children phases should be removed
        assert!(tree["children"][0].get("phase").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child1")
        );

        assert!(tree["children"][1].get("phase").is_none());
        assert_eq!(
            tree["children"][1].get("name").unwrap().as_str(),
            Some("Child2")
        );
    }

    #[test]
    fn test_remove_phase_deeply_nested() {
        let mut tree = json!({
            "document": {
                "phase": {
                    "__enum__": "NodePhase",
                    "value": "CREATED"
                },
                "children": [
                    {
                        "phase": {
                            "__enum__": "NodePhase",
                            "value": "CREATED"
                        },
                        "children": [
                            {
                                "phase": {
                                    "__enum__": "NodePhase",
                                    "value": "CREATED"
                                },
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_phase_fields(&mut tree).unwrap();

        // All phases should be removed at all levels
        assert!(tree["document"].get("phase").is_none());
        assert!(tree["document"]["children"][0].get("phase").is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("phase")
            .is_none());

        // Other fields should be preserved
        assert_eq!(
            tree["document"]["children"][0]["children"][0]
                .get("name")
                .unwrap()
                .as_str(),
            Some("DeepChild")
        );
    }

    #[test]
    fn test_remove_phase_missing() {
        let mut tree = json!({
            "name": "Rectangle",
            "visible": true,
            "x": 10,
            "y": 20
        });

        remove_phase_fields(&mut tree).unwrap();

        // Tree without phase should be unchanged
        assert!(tree.get("phase").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));
    }

    #[test]
    fn test_remove_phase_preserves_other_fields() {
        let mut tree = json!({
            "name": "Frame",
            "phase": {
                "__enum__": "NodePhase",
                "value": "CREATED"
            },
            "type": "FRAME",
            "opacity": 1.0,
            "visible": true,
            "x": 100,
            "y": 200
        });

        remove_phase_fields(&mut tree).unwrap();

        // Only phase should be removed
        assert!(tree.get("phase").is_none());

        // All other fields preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(200));
    }

    #[test]
    fn test_remove_phase_in_arrays() {
        let mut tree = json!({
            "items": [
                {
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "CREATED"
                    },
                    "name": "Item1"
                },
                {
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "CREATED"
                    },
                    "name": "Item2"
                }
            ]
        });

        remove_phase_fields(&mut tree).unwrap();

        // All phases in array should be removed
        assert!(tree["items"][0].get("phase").is_none());
        assert_eq!(
            tree["items"][0].get("name").unwrap().as_str(),
            Some("Item1")
        );

        assert!(tree["items"][1].get("phase").is_none());
        assert_eq!(
            tree["items"][1].get("name").unwrap().as_str(),
            Some("Item2")
        );
    }

    #[test]
    fn test_remove_phase_mixed_objects() {
        let mut tree = json!({
            "name": "Root",
            "phase": {
                "__enum__": "NodePhase",
                "value": "CREATED"
            },
            "properties": {
                "width": 100,
                "height": 200
            },
            "children": [
                {
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "CREATED"
                    },
                    "name": "Child"
                }
            ]
        });

        remove_phase_fields(&mut tree).unwrap();

        // Root phase removed
        assert!(tree.get("phase").is_none());

        // Properties object unchanged (no phase)
        assert_eq!(tree["properties"]["width"].as_i64(), Some(100));
        assert_eq!(tree["properties"]["height"].as_i64(), Some(200));

        // Child phase removed
        assert!(tree["children"][0].get("phase").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child")
        );
    }

    #[test]
    fn test_remove_phase_empty_object() {
        let mut tree = json!({});

        remove_phase_fields(&mut tree).unwrap();

        // Empty object should remain empty
        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_remove_phase_primitives() {
        let mut tree = json!(true);

        remove_phase_fields(&mut tree).unwrap();

        // Primitive values should be unchanged
        assert_eq!(tree.as_bool(), Some(true));
    }
}
