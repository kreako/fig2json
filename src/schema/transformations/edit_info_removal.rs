use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove editInfo fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes all "editInfo" fields.
/// These fields contain version control metadata (createdAt, lastEditedAt, userId)
/// that are not needed for HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all editInfo fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_edit_info_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "editInfo": {
///         "createdAt": 1761413476,
///         "lastEditedAt": 1761413532,
///         "userId": "1106160570506540696"
///     },
///     "visible": true
/// });
/// remove_edit_info_fields(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_edit_info_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove editInfo fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the "editInfo" field if it exists
            map.remove("editInfo");

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
    fn test_remove_edit_info_simple() {
        let mut tree = json!({
            "name": "Rectangle",
            "editInfo": {
                "createdAt": 1761413476,
                "lastEditedAt": 1761413532,
                "userId": "1106160570506540696"
            },
            "visible": true
        });

        remove_edit_info_fields(&mut tree).unwrap();

        assert!(tree.get("editInfo").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_edit_info_nested() {
        let mut tree = json!({
            "name": "Root",
            "editInfo": {
                "createdAt": 0,
                "lastEditedAt": 1761414263,
                "userId": "1106160570506540696"
            },
            "children": [
                {
                    "name": "Child1",
                    "editInfo": {
                        "createdAt": 1761413389,
                        "lastEditedAt": 1761414263,
                        "userId": "1106160570506540696"
                    }
                },
                {
                    "name": "Child2",
                    "editInfo": {
                        "createdAt": 1761414252,
                        "lastEditedAt": 1761414252,
                        "userId": "1106160570506540696"
                    }
                }
            ]
        });

        remove_edit_info_fields(&mut tree).unwrap();

        // Root editInfo should be removed
        assert!(tree.get("editInfo").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Root"));

        // Children editInfo should be removed
        assert!(tree["children"][0].get("editInfo").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child1")
        );

        assert!(tree["children"][1].get("editInfo").is_none());
        assert_eq!(
            tree["children"][1].get("name").unwrap().as_str(),
            Some("Child2")
        );
    }

    #[test]
    fn test_remove_edit_info_deeply_nested() {
        let mut tree = json!({
            "document": {
                "editInfo": {
                    "createdAt": 0,
                    "lastEditedAt": 1000,
                    "userId": "user1"
                },
                "children": [
                    {
                        "editInfo": {
                            "createdAt": 500,
                            "lastEditedAt": 800,
                            "userId": "user2"
                        },
                        "children": [
                            {
                                "editInfo": {
                                    "createdAt": 600,
                                    "lastEditedAt": 700,
                                    "userId": "user3"
                                },
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_edit_info_fields(&mut tree).unwrap();

        // All editInfo should be removed at all levels
        assert!(tree["document"].get("editInfo").is_none());
        assert!(tree["document"]["children"][0].get("editInfo").is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("editInfo")
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
    fn test_remove_edit_info_missing() {
        let mut tree = json!({
            "name": "Rectangle",
            "visible": true,
            "x": 10,
            "y": 20
        });

        remove_edit_info_fields(&mut tree).unwrap();

        // Tree without editInfo should be unchanged
        assert!(tree.get("editInfo").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));
    }

    #[test]
    fn test_remove_edit_info_preserves_other_fields() {
        let mut tree = json!({
            "name": "Frame",
            "editInfo": {
                "createdAt": 1761413389,
                "lastEditedAt": 1761414263,
                "userId": "1106160570506540696"
            },
            "type": "FRAME",
            "opacity": 1.0,
            "visible": true,
            "x": 100,
            "y": 200
        });

        remove_edit_info_fields(&mut tree).unwrap();

        // Only editInfo should be removed
        assert!(tree.get("editInfo").is_none());

        // All other fields preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(200));
    }

    #[test]
    fn test_remove_edit_info_in_arrays() {
        let mut tree = json!({
            "items": [
                {
                    "editInfo": {
                        "createdAt": 1000,
                        "lastEditedAt": 2000,
                        "userId": "user1"
                    },
                    "name": "Item1"
                },
                {
                    "editInfo": {
                        "createdAt": 3000,
                        "lastEditedAt": 4000,
                        "userId": "user2"
                    },
                    "name": "Item2"
                }
            ]
        });

        remove_edit_info_fields(&mut tree).unwrap();

        // All editInfo in array should be removed
        assert!(tree["items"][0].get("editInfo").is_none());
        assert_eq!(
            tree["items"][0].get("name").unwrap().as_str(),
            Some("Item1")
        );

        assert!(tree["items"][1].get("editInfo").is_none());
        assert_eq!(
            tree["items"][1].get("name").unwrap().as_str(),
            Some("Item2")
        );
    }

    #[test]
    fn test_remove_edit_info_mixed_objects() {
        let mut tree = json!({
            "name": "Root",
            "editInfo": {
                "createdAt": 0,
                "lastEditedAt": 1000,
                "userId": "root_user"
            },
            "properties": {
                "width": 100,
                "height": 200
            },
            "children": [
                {
                    "editInfo": {
                        "createdAt": 500,
                        "lastEditedAt": 800,
                        "userId": "child_user"
                    },
                    "name": "Child"
                }
            ]
        });

        remove_edit_info_fields(&mut tree).unwrap();

        // Root editInfo removed
        assert!(tree.get("editInfo").is_none());

        // Properties object unchanged (no editInfo)
        assert_eq!(tree["properties"]["width"].as_i64(), Some(100));
        assert_eq!(tree["properties"]["height"].as_i64(), Some(200));

        // Child editInfo removed
        assert!(tree["children"][0].get("editInfo").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child")
        );
    }

    #[test]
    fn test_remove_edit_info_empty_object() {
        let mut tree = json!({});

        remove_edit_info_fields(&mut tree).unwrap();

        // Empty object should remain empty
        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_remove_edit_info_primitives() {
        let mut tree = json!(42);

        remove_edit_info_fields(&mut tree).unwrap();

        // Primitive values should be unchanged
        assert_eq!(tree.as_i64(), Some(42));
    }
}
