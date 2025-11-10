use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove objects that only contain a visible property from the JSON tree
///
/// Recursively traverses the JSON tree and removes objects that have only one key
/// named "visible". These objects typically appear in Figma's symbolOverrides arrays
/// and serve to hide/show elements without providing other meaningful data.
///
/// Objects with only `visible` are removed from:
/// - Arrays (the visible-only object elements are filtered out)
/// - Object values (the key-value pair is removed if the value only has `visible`)
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all visible-only objects
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_visible_only_objects;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "symbolOverrides": [
///         {"visible": false},
///         {"textData": {"characters": "Hello"}},
///         {"visible": true}
///     ]
/// });
/// remove_visible_only_objects(&mut tree).unwrap();
/// // symbolOverrides now only has the textData object
/// ```
pub fn remove_visible_only_objects(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree);
    Ok(())
}

/// Recursively remove visible-only objects from a JSON value
fn transform_recursive(value: &mut JsonValue) {
    match value {
        JsonValue::Object(map) => {
            // First, recurse into all values
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in &keys {
                if let Some(val) = map.get_mut(key) {
                    transform_recursive(val);
                }
            }

            // Then remove any keys whose values are visible-only objects
            map.retain(|_, v| !is_visible_only_object(v));
        }
        JsonValue::Array(arr) => {
            // First, recurse into array elements
            for val in arr.iter_mut() {
                transform_recursive(val);
            }

            // Then filter out visible-only objects from the array
            arr.retain(|v| !is_visible_only_object(v));
        }
        _ => {
            // Primitives - nothing to do
        }
    }
}

/// Check if a JSON value is an object with only a "visible" key
fn is_visible_only_object(value: &JsonValue) -> bool {
    match value {
        JsonValue::Object(map) => {
            map.len() == 1 && map.contains_key("visible")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_visible_only_from_array() {
        let mut tree = json!({
            "symbolOverrides": [
                {"visible": false},
                {"textData": {"characters": "Hello"}},
                {"visible": true}
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
        assert_eq!(overrides.len(), 1);
        assert!(overrides[0].get("textData").is_some());
    }

    #[test]
    fn test_remove_visible_only_object_field() {
        let mut tree = json!({
            "name": "Shape",
            "metadata": {"visible": false},
            "opacity": 1.0
        });

        remove_visible_only_objects(&mut tree).unwrap();

        assert!(tree.get("metadata").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_preserve_objects_with_visible_and_other_fields() {
        let mut tree = json!({
            "symbolOverrides": [
                {"visible": false, "opacity": 0.5},
                {"visible": true, "textData": {"characters": "Test"}},
                {"visible": false}
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
        assert_eq!(overrides.len(), 2);
        assert!(overrides[0].get("visible").is_some());
        assert!(overrides[0].get("opacity").is_some());
        assert!(overrides[1].get("visible").is_some());
        assert!(overrides[1].get("textData").is_some());
    }

    #[test]
    fn test_nested_visible_only_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "data": {"visible": true}
                },
                {
                    "name": "Child2",
                    "overrides": [
                        {"visible": false},
                        {"opacity": 0.5}
                    ]
                }
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        assert!(tree["children"][0].get("data").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        let overrides = tree["children"][1]["overrides"].as_array().unwrap();
        assert_eq!(overrides.len(), 1);
        assert!(overrides[0].get("opacity").is_some());
    }

    #[test]
    fn test_array_of_visible_only_objects() {
        let mut tree = json!({
            "items": [
                {"visible": false},
                {"visible": true},
                {"visible": false}
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let items = tree.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_mixed_array() {
        let mut tree = json!({
            "items": [
                {"visible": false},
                {"name": "A"},
                {"visible": true},
                {"name": "B", "visible": false},
                {"visible": false}
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let items = tree.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["name"].as_str(), Some("A"));
        assert_eq!(items[1]["name"].as_str(), Some("B"));
        // Second item should still have visible since it has other fields too
        assert!(items[1].get("visible").is_some());
    }

    #[test]
    fn test_no_visible_only_objects() {
        let mut tree = json!({
            "name": "Rectangle",
            "visible": true,
            "children": [
                {"name": "Child1", "visible": false},
                {"name": "Child2"}
            ]
        });

        let original = tree.clone();
        remove_visible_only_objects(&mut tree).unwrap();

        // Tree should be unchanged since all visible fields have other fields too
        assert_eq!(tree, original);
    }

    #[test]
    fn test_real_world_roles_members_case() {
        // From archives/roles-members.json line 203
        let mut tree = json!({
            "symbolData": {
                "symbolOverrides": [
                    {
                        "textData": {
                            "characters": "Roles"
                        }
                    },
                    {
                        "textData": {
                            "characters": "Members"
                        }
                    },
                    {
                        "textData": {
                            "characters": "Audit"
                        }
                    },
                    {
                        "visible": false
                    },
                    {
                        "overrideLevel": 1,
                        "textData": {
                            "characters": "Commands"
                        }
                    }
                ],
                "uniformScaleFactor": 1.0
            }
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let overrides = tree["symbolData"]["symbolOverrides"].as_array().unwrap();
        assert_eq!(overrides.len(), 4);
        // Verify the visible-only object was removed
        assert!(overrides.iter().all(|o| o.get("textData").is_some() || o.get("overrideLevel").is_some()));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "visibleOnly": {"visible": true},
                        "data": "value"
                    },
                    "alsoVisibleOnly": {"visible": false}
                }
            }
        });

        remove_visible_only_objects(&mut tree).unwrap();

        assert!(tree["level1"]["level2"]["level3"].get("visibleOnly").is_none());
        assert_eq!(
            tree["level1"]["level2"]["level3"]["data"].as_str(),
            Some("value")
        );
        assert!(tree["level1"]["level2"].get("alsoVisibleOnly").is_none());
    }

    #[test]
    fn test_preserve_empty_objects() {
        let mut tree = json!({
            "name": "Test",
            "empty": {},
            "visibleOnly": {"visible": false}
        });

        remove_visible_only_objects(&mut tree).unwrap();

        // Empty objects should be preserved, only visible-only objects are removed
        assert!(tree.get("empty").is_some());
        assert!(tree.get("visibleOnly").is_none());
    }

    #[test]
    fn test_visible_with_different_values() {
        let mut tree = json!({
            "items": [
                {"visible": false},
                {"visible": true},
                {"visible": 0},
                {"visible": 1},
                {"visible": null}
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let items = tree.get("items").unwrap().as_array().unwrap();
        // All should be removed regardless of the visible value
        assert_eq!(items.len(), 0);
    }
}
