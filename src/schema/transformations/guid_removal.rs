use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove guid fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes all "guid" fields.
/// These fields contain internal Figma identifiers (localID, sessionID) that
/// are not needed for HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all guid fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_guid_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "guid": {
///         "localID": 3,
///         "sessionID": 1
///     },
///     "visible": true
/// });
/// remove_guid_fields(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_guid_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove guid fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the "guid" field if it exists
            map.remove("guid");

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
    fn test_remove_guid_simple() {
        let mut tree = json!({
            "name": "Rectangle",
            "guid": {
                "localID": 3,
                "sessionID": 1
            },
            "visible": true
        });

        remove_guid_fields(&mut tree).unwrap();

        assert!(tree.get("guid").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_guid_nested() {
        let mut tree = json!({
            "name": "Root",
            "guid": {
                "localID": 0,
                "sessionID": 0
            },
            "children": [
                {
                    "name": "Child1",
                    "guid": {
                        "localID": 1,
                        "sessionID": 0
                    }
                },
                {
                    "name": "Child2",
                    "guid": {
                        "localID": 2,
                        "sessionID": 1
                    }
                }
            ]
        });

        remove_guid_fields(&mut tree).unwrap();

        // Root guid should be removed
        assert!(tree.get("guid").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Root"));

        // Children guids should be removed
        assert!(tree["children"][0].get("guid").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child1")
        );

        assert!(tree["children"][1].get("guid").is_none());
        assert_eq!(
            tree["children"][1].get("name").unwrap().as_str(),
            Some("Child2")
        );
    }

    #[test]
    fn test_remove_guid_deeply_nested() {
        let mut tree = json!({
            "document": {
                "guid": {
                    "localID": 0,
                    "sessionID": 0
                },
                "children": [
                    {
                        "guid": {
                            "localID": 1,
                            "sessionID": 0
                        },
                        "children": [
                            {
                                "guid": {
                                    "localID": 2,
                                    "sessionID": 0
                                },
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_guid_fields(&mut tree).unwrap();

        // All guids should be removed at all levels
        assert!(tree["document"].get("guid").is_none());
        assert!(tree["document"]["children"][0].get("guid").is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("guid")
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
    fn test_remove_guid_missing() {
        let mut tree = json!({
            "name": "Rectangle",
            "visible": true,
            "x": 10,
            "y": 20
        });

        remove_guid_fields(&mut tree).unwrap();

        // Tree without guid should be unchanged
        assert!(tree.get("guid").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));
    }

    #[test]
    fn test_remove_guid_preserves_other_fields() {
        let mut tree = json!({
            "name": "Frame",
            "guid": {
                "localID": 5,
                "sessionID": 2
            },
            "type": "FRAME",
            "opacity": 1.0,
            "visible": true,
            "x": 100,
            "y": 200
        });

        remove_guid_fields(&mut tree).unwrap();

        // Only guid should be removed
        assert!(tree.get("guid").is_none());

        // All other fields preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(200));
    }

    #[test]
    fn test_remove_guid_in_arrays() {
        let mut tree = json!({
            "items": [
                {
                    "guid": {
                        "localID": 1,
                        "sessionID": 0
                    },
                    "name": "Item1"
                },
                {
                    "guid": {
                        "localID": 2,
                        "sessionID": 0
                    },
                    "name": "Item2"
                }
            ]
        });

        remove_guid_fields(&mut tree).unwrap();

        // All guids in array should be removed
        assert!(tree["items"][0].get("guid").is_none());
        assert_eq!(
            tree["items"][0].get("name").unwrap().as_str(),
            Some("Item1")
        );

        assert!(tree["items"][1].get("guid").is_none());
        assert_eq!(
            tree["items"][1].get("name").unwrap().as_str(),
            Some("Item2")
        );
    }

    #[test]
    fn test_remove_guid_mixed_objects() {
        let mut tree = json!({
            "name": "Root",
            "guid": {
                "localID": 0,
                "sessionID": 0
            },
            "properties": {
                "width": 100,
                "height": 200
            },
            "children": [
                {
                    "guid": {
                        "localID": 1,
                        "sessionID": 0
                    },
                    "name": "Child"
                }
            ]
        });

        remove_guid_fields(&mut tree).unwrap();

        // Root guid removed
        assert!(tree.get("guid").is_none());

        // Properties object unchanged (no guid)
        assert_eq!(tree["properties"]["width"].as_i64(), Some(100));
        assert_eq!(tree["properties"]["height"].as_i64(), Some(200));

        // Child guid removed
        assert!(tree["children"][0].get("guid").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child")
        );
    }

    #[test]
    fn test_remove_guid_empty_object() {
        let mut tree = json!({});

        remove_guid_fields(&mut tree).unwrap();

        // Empty object should remain empty
        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_remove_guid_primitives() {
        let mut tree = json!("string value");

        remove_guid_fields(&mut tree).unwrap();

        // Primitive values should be unchanged
        assert_eq!(tree.as_str(), Some("string value"));
    }
}
