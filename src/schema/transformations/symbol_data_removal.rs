use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove symbolData fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes all "symbolData" fields.
/// These fields contain Figma component instance metadata including symbolID,
/// symbolOverrides, and uniformScaleFactor that are not needed for HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all symbolData fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_symbol_data;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Button",
///     "type": "INSTANCE",
///     "symbolData": {
///         "symbolID": {
///             "localID": 123,
///             "sessionID": 456
///         },
///         "uniformScaleFactor": 1.0
///     },
///     "visible": true
/// });
/// remove_symbol_data(&mut tree).unwrap();
/// // tree now has only "name", "type", and "visible" fields
/// ```
pub fn remove_symbol_data(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove symbolData fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the "symbolData" field if it exists
            map.remove("symbolData");

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
    fn test_remove_symbol_data_simple() {
        let mut tree = json!({
            "name": "Button",
            "type": "INSTANCE",
            "symbolData": {
                "symbolID": {
                    "localID": 123,
                    "sessionID": 456
                },
                "uniformScaleFactor": 1.0
            },
            "visible": true
        });

        remove_symbol_data(&mut tree).unwrap();

        assert!(tree.get("symbolData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("INSTANCE"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_symbol_data_with_overrides() {
        let mut tree = json!({
            "name": "Component",
            "symbolData": {
                "symbolID": {
                    "localID": 100,
                    "sessionID": 200
                },
                "symbolOverrides": [
                    {
                        "guidPath": {
                            "guids": [
                                {"localID": 1, "sessionID": 1}
                            ]
                        },
                        "visible": false
                    }
                ],
                "uniformScaleFactor": 1.0
            }
        });

        remove_symbol_data(&mut tree).unwrap();

        assert!(tree.get("symbolData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Component"));
    }

    #[test]
    fn test_remove_symbol_data_nested() {
        let mut tree = json!({
            "name": "Root",
            "symbolData": {
                "symbolID": {"localID": 1, "sessionID": 1}
            },
            "children": [
                {
                    "name": "Child1",
                    "symbolData": {
                        "symbolID": {"localID": 2, "sessionID": 1}
                    }
                },
                {
                    "name": "Child2",
                    "symbolData": {
                        "symbolID": {"localID": 3, "sessionID": 1}
                    }
                }
            ]
        });

        remove_symbol_data(&mut tree).unwrap();

        assert!(tree.get("symbolData").is_none());
        assert!(tree["children"][0].get("symbolData").is_none());
        assert!(tree["children"][1].get("symbolData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Root"));
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_remove_symbol_data_deeply_nested() {
        let mut tree = json!({
            "document": {
                "symbolData": {
                    "symbolID": {"localID": 0, "sessionID": 0}
                },
                "children": [
                    {
                        "symbolData": {
                            "symbolID": {"localID": 1, "sessionID": 0}
                        },
                        "children": [
                            {
                                "symbolData": {
                                    "symbolID": {"localID": 2, "sessionID": 0}
                                },
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_symbol_data(&mut tree).unwrap();

        assert!(tree["document"].get("symbolData").is_none());
        assert!(tree["document"]["children"][0].get("symbolData").is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("symbolData")
            .is_none());
        assert_eq!(
            tree["document"]["children"][0]["children"][0]
                .get("name")
                .unwrap()
                .as_str(),
            Some("DeepChild")
        );
    }

    #[test]
    fn test_remove_symbol_data_missing() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "visible": true,
            "x": 10,
            "y": 20
        });

        remove_symbol_data(&mut tree).unwrap();

        assert!(tree.get("symbolData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_symbol_data_preserves_other_fields() {
        let mut tree = json!({
            "name": "Instance",
            "type": "INSTANCE",
            "symbolData": {
                "symbolID": {"localID": 5, "sessionID": 2},
                "uniformScaleFactor": 1.5
            },
            "opacity": 0.8,
            "visible": true,
            "transform": {
                "x": 100,
                "y": 200
            }
        });

        remove_symbol_data(&mut tree).unwrap();

        assert!(tree.get("symbolData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Instance"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("INSTANCE"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.8));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree["transform"]["x"].as_i64(), Some(100));
    }

    #[test]
    fn test_remove_symbol_data_in_arrays() {
        let mut tree = json!({
            "instances": [
                {
                    "symbolData": {
                        "symbolID": {"localID": 1, "sessionID": 0}
                    },
                    "name": "Instance1"
                },
                {
                    "symbolData": {
                        "symbolID": {"localID": 2, "sessionID": 0}
                    },
                    "name": "Instance2"
                }
            ]
        });

        remove_symbol_data(&mut tree).unwrap();

        assert!(tree["instances"][0].get("symbolData").is_none());
        assert_eq!(
            tree["instances"][0].get("name").unwrap().as_str(),
            Some("Instance1")
        );
        assert!(tree["instances"][1].get("symbolData").is_none());
        assert_eq!(
            tree["instances"][1].get("name").unwrap().as_str(),
            Some("Instance2")
        );
    }

    #[test]
    fn test_remove_symbol_data_empty_object() {
        let mut tree = json!({});

        remove_symbol_data(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_remove_symbol_data_primitives() {
        let mut tree = json!("string value");

        remove_symbol_data(&mut tree).unwrap();

        assert_eq!(tree.as_str(), Some("string value"));
    }

    #[test]
    fn test_remove_symbol_data_mixed_types() {
        let mut tree = json!({
            "name": "Root",
            "symbolData": {
                "symbolID": {"localID": 0, "sessionID": 0}
            },
            "properties": {
                "width": 100,
                "height": 200
            },
            "children": [
                {
                    "symbolData": {
                        "symbolID": {"localID": 1, "sessionID": 0}
                    },
                    "name": "Child"
                }
            ]
        });

        remove_symbol_data(&mut tree).unwrap();

        assert!(tree.get("symbolData").is_none());
        assert_eq!(tree["properties"]["width"].as_i64(), Some(100));
        assert!(tree["children"][0].get("symbolData").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child")
        );
    }
}
