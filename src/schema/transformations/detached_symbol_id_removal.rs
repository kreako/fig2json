use crate::error::Result;
use serde_json::Value as JsonValue;

/// Removes Figma component instance metadata from the document tree.
///
/// This transformation removes the `detachedSymbolId` field, which contains
/// references to Figma component instances that have been detached from their
/// main component. The field typically contains an `assetRef` object with
/// `key` and `version` properties.
///
/// This metadata is specific to Figma's component system and is not needed
/// for HTML/CSS rendering.
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_detached_symbol_id;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "detachedSymbolId": {
///         "assetRef": {
///             "key": "b12947c871f268e97f688eb784bcf92431d9b6df",
///             "version": "186:107"
///         }
///     },
///     "type": "FRAME"
/// });
///
/// remove_detached_symbol_id(&mut tree).unwrap();
///
/// assert!(tree.get("detachedSymbolId").is_none());
/// assert!(tree.get("type").is_some());
/// ```
pub fn remove_detached_symbol_id(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove detachedSymbolId
            map.remove("detachedSymbolId");

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
    fn test_removes_detached_symbol_id_with_asset_ref() {
        let mut tree = json!({
            "name": "Frame",
            "detachedSymbolId": {
                "assetRef": {
                    "key": "b12947c871f268e97f688eb784bcf92431d9b6df",
                    "version": "186:107"
                }
            },
            "type": "FRAME"
        });

        remove_detached_symbol_id(&mut tree).unwrap();

        assert!(tree.get("detachedSymbolId").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
    }

    #[test]
    fn test_removes_detached_symbol_id_empty_object() {
        let mut tree = json!({
            "name": "Frame",
            "detachedSymbolId": {},
            "type": "FRAME"
        });

        remove_detached_symbol_id(&mut tree).unwrap();

        assert!(tree.get("detachedSymbolId").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "children": [
                {
                    "name": "Child1",
                    "detachedSymbolId": {
                        "assetRef": {
                            "key": "key1",
                            "version": "1:1"
                        }
                    }
                },
                {
                    "name": "Child2",
                    "detachedSymbolId": {
                        "assetRef": {
                            "key": "key2",
                            "version": "2:2"
                        }
                    }
                }
            ]
        });

        remove_detached_symbol_id(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("detachedSymbolId").is_none());
        assert!(children[1].get("detachedSymbolId").is_none());
        assert_eq!(children[0].get("name").unwrap().as_str(), Some("Child1"));
        assert_eq!(children[1].get("name").unwrap().as_str(), Some("Child2"));
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "detachedSymbolId": {
                "assetRef": {
                    "key": "root-key",
                    "version": "1:1"
                }
            },
            "children": [
                {
                    "name": "Level1",
                    "children": [
                        {
                            "name": "Level2",
                            "detachedSymbolId": {
                                "assetRef": {
                                    "key": "level2-key",
                                    "version": "3:3"
                                }
                            }
                        }
                    ]
                }
            ]
        });

        remove_detached_symbol_id(&mut tree).unwrap();

        assert!(tree.get("detachedSymbolId").is_none());
        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("detachedSymbolId").is_none());
        assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
    }

    #[test]
    fn test_handles_missing_detached_symbol_id() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_detached_symbol_id(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("type").is_some());
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_detached_symbol_id(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "home-indicator",
            "type": "FRAME",
            "detachedSymbolId": {
                "assetRef": {
                    "key": "b12947c871f268e97f688eb784bcf92431d9b6df",
                    "version": "186:107"
                }
            },
            "scrollBehavior": "FIXED_WHEN_CHILD_OF_SCROLLING_FRAME",
            "size": {
                "x": 375.0,
                "y": 15.0119047164917
            }
        });

        remove_detached_symbol_id(&mut tree).unwrap();

        assert!(tree.get("detachedSymbolId").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("home-indicator"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert!(tree.get("scrollBehavior").is_some());
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_handles_multiple_occurrences_in_array() {
        let mut tree = json!({
            "children": [
                {
                    "name": "A",
                    "detachedSymbolId": {
                        "assetRef": {"key": "a", "version": "1:1"}
                    }
                },
                {"name": "B"},
                {
                    "name": "C",
                    "detachedSymbolId": {
                        "assetRef": {"key": "c", "version": "3:3"}
                    }
                }
            ]
        });

        remove_detached_symbol_id(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        for child in children {
            assert!(child.get("detachedSymbolId").is_none());
            assert!(child.get("name").is_some());
        }
    }

    #[test]
    fn test_removes_complex_nested_asset_ref() {
        let mut tree = json!({
            "name": "tabbar",
            "detachedSymbolId": {
                "assetRef": {
                    "key": "4e230a17eeb81f100aa84a3c3b4734692e7a3b38",
                    "version": "3912:618"
                }
            },
            "type": "FRAME"
        });

        remove_detached_symbol_id(&mut tree).unwrap();

        assert!(tree.get("detachedSymbolId").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("tabbar"));
    }
}
