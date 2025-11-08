use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove visible field when it has the default value true
///
/// Recursively traverses the JSON tree and removes "visible" fields that have
/// the value true. Since true is the default visibility in both Figma and CSS,
/// omitting it reduces output size without losing information.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all default visible fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_default_visible;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "visible": true,
///     "opacity": 0.5
/// });
/// remove_default_visible(&mut tree).unwrap();
/// // tree now has only "name" and "opacity" fields
/// ```
pub fn remove_default_visible(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove default visible fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check if visible exists and is true
            if let Some(visible) = map.get("visible") {
                if let Some(b) = visible.as_bool() {
                    if b {
                        map.remove("visible");
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_default_visible() {
        let mut tree = json!({
            "name": "Shape",
            "visible": true,
            "opacity": 0.5
        });

        remove_default_visible(&mut tree).unwrap();

        assert!(tree.get("visible").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.5));
    }

    #[test]
    fn test_preserve_visible_false() {
        let mut tree = json!({
            "name": "Shape",
            "visible": false,
            "opacity": 1.0
        });

        remove_default_visible(&mut tree).unwrap();

        // visible: false should be preserved
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(false));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
    }

    #[test]
    fn test_no_visible() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_visible(&mut tree).unwrap();

        // Tree without visible should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("visible").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "visible": true
                },
                {
                    "name": "Child2",
                    "visible": false
                }
            ]
        });

        remove_default_visible(&mut tree).unwrap();

        // visible: true removed, visible: false preserved
        assert!(tree["children"][0].get("visible").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(tree["children"][1]["visible"].as_bool(), Some(false));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_visible_in_paints() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "SOLID",
                    "visible": true,
                    "color": "#ff0000"
                },
                {
                    "type": "GRADIENT",
                    "visible": false,
                    "color": "#00ff00"
                }
            ]
        });

        remove_default_visible(&mut tree).unwrap();

        // visible: true removed from first paint
        assert!(tree["fillPaints"][0].get("visible").is_none());
        assert_eq!(tree["fillPaints"][0]["type"].as_str(), Some("SOLID"));

        // visible: false preserved in second paint
        assert_eq!(tree["fillPaints"][1]["visible"].as_bool(), Some(false));
        assert_eq!(tree["fillPaints"][1]["type"].as_str(), Some("GRADIENT"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "visible": true,
                        "fillPaints": [
                            {
                                "type": "SOLID",
                                "visible": true
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_visible(&mut tree).unwrap();

        // All visible: true should be removed at all levels
        let frame = &tree["document"]["children"][0];
        assert!(frame.get("visible").is_none());
        assert!(frame["fillPaints"][0].get("visible").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_multiple_default_visible() {
        let mut tree = json!({
            "children": [
                {"visible": true, "name": "A"},
                {"visible": true, "name": "B"},
                {"visible": true, "name": "C"}
            ]
        });

        remove_default_visible(&mut tree).unwrap();

        // All visible: true should be removed
        assert!(tree["children"][0].get("visible").is_none());
        assert!(tree["children"][1].get("visible").is_none());
        assert!(tree["children"][2].get("visible").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_multiple_false_visible() {
        let mut tree = json!({
            "children": [
                {"visible": false, "name": "A"},
                {"visible": false, "name": "B"},
                {"visible": false, "name": "C"}
            ]
        });

        remove_default_visible(&mut tree).unwrap();

        // All visible: false should be preserved
        assert_eq!(tree["children"][0]["visible"].as_bool(), Some(false));
        assert_eq!(tree["children"][1]["visible"].as_bool(), Some(false));
        assert_eq!(tree["children"][2]["visible"].as_bool(), Some(false));
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_mixed_visible_values() {
        let mut tree = json!({
            "children": [
                {"visible": true, "name": "A"},
                {"visible": false, "name": "B"},
                {"visible": true, "name": "C"},
                {"visible": false, "name": "D"}
            ]
        });

        remove_default_visible(&mut tree).unwrap();

        // true removed, false preserved
        assert!(tree["children"][0].get("visible").is_none());
        assert_eq!(tree["children"][1]["visible"].as_bool(), Some(false));
        assert!(tree["children"][2].get("visible").is_none());
        assert_eq!(tree["children"][3]["visible"].as_bool(), Some(false));
    }

    #[test]
    fn test_visible_string_not_touched() {
        let mut tree = json!({
            "name": "Test",
            "visible": "true"
        });

        remove_default_visible(&mut tree).unwrap();

        // String visible should not be touched
        assert_eq!(tree.get("visible").unwrap().as_str(), Some("true"));
    }
}
