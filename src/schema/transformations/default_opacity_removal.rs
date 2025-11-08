use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove opacity field when it has the default value 1.0
///
/// Recursively traverses the JSON tree and removes "opacity" fields that have
/// the value 1.0. Since 1.0 is the default opacity in both Figma and CSS,
/// omitting it reduces output size without losing information.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all default opacity fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_default_opacity;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "opacity": 1.0,
///     "visible": true
/// });
/// remove_default_opacity(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_default_opacity(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove default opacity fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check if opacity exists and is 1.0
            if let Some(opacity) = map.get("opacity") {
                if let Some(n) = opacity.as_f64() {
                    // Use epsilon comparison for floating point
                    if (n - 1.0).abs() < f64::EPSILON {
                        map.remove("opacity");
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
    fn test_remove_default_opacity() {
        let mut tree = json!({
            "name": "Shape",
            "opacity": 1.0,
            "visible": true
        });

        remove_default_opacity(&mut tree).unwrap();

        assert!(tree.get("opacity").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_preserve_non_default_opacity() {
        let mut tree = json!({
            "name": "Shape",
            "opacity": 0.5,
            "visible": true
        });

        remove_default_opacity(&mut tree).unwrap();

        // Non-1.0 opacity should be preserved
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.5));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
    }

    #[test]
    fn test_preserve_zero_opacity() {
        let mut tree = json!({
            "name": "Shape",
            "opacity": 0.0
        });

        remove_default_opacity(&mut tree).unwrap();

        // Zero opacity should be preserved
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.0));
    }

    #[test]
    fn test_preserve_various_opacities() {
        let opacities = vec![0.0, 0.25, 0.5, 0.75, 0.9, 0.99];

        for opacity_value in opacities {
            let mut tree = json!({
                "opacity": opacity_value
            });

            remove_default_opacity(&mut tree).unwrap();

            // All non-1.0 opacities should be preserved
            assert_eq!(
                tree.get("opacity").unwrap().as_f64(),
                Some(opacity_value)
            );
        }
    }

    #[test]
    fn test_no_opacity() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_opacity(&mut tree).unwrap();

        // Tree without opacity should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("opacity").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "opacity": 1.0
                },
                {
                    "name": "Child2",
                    "opacity": 0.7
                }
            ]
        });

        remove_default_opacity(&mut tree).unwrap();

        // opacity 1.0 removed, 0.7 preserved
        assert!(tree["children"][0].get("opacity").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(tree["children"][1]["opacity"].as_f64(), Some(0.7));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_opacity_in_paints() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "SOLID",
                    "opacity": 1.0,
                    "color": "#ff0000"
                },
                {
                    "type": "GRADIENT",
                    "opacity": 0.8,
                    "color": "#00ff00"
                }
            ]
        });

        remove_default_opacity(&mut tree).unwrap();

        // opacity 1.0 removed from first paint
        assert!(tree["fillPaints"][0].get("opacity").is_none());
        assert_eq!(tree["fillPaints"][0]["type"].as_str(), Some("SOLID"));

        // opacity 0.8 preserved in second paint
        assert_eq!(tree["fillPaints"][1]["opacity"].as_f64(), Some(0.8));
        assert_eq!(tree["fillPaints"][1]["type"].as_str(), Some("GRADIENT"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "opacity": 1.0,
                        "fillPaints": [
                            {
                                "type": "SOLID",
                                "opacity": 1.0
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_opacity(&mut tree).unwrap();

        // All opacity 1.0 should be removed at all levels
        let frame = &tree["document"]["children"][0];
        assert!(frame.get("opacity").is_none());
        assert!(frame["fillPaints"][0].get("opacity").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_multiple_default_opacities() {
        let mut tree = json!({
            "children": [
                {"opacity": 1.0, "name": "A"},
                {"opacity": 1.0, "name": "B"},
                {"opacity": 1.0, "name": "C"}
            ]
        });

        remove_default_opacity(&mut tree).unwrap();

        // All opacity 1.0 should be removed
        assert!(tree["children"][0].get("opacity").is_none());
        assert!(tree["children"][1].get("opacity").is_none());
        assert!(tree["children"][2].get("opacity").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_opacity_as_integer() {
        let mut tree = json!({
            "name": "Shape",
            "opacity": 1
        });

        remove_default_opacity(&mut tree).unwrap();

        // Integer 1 should also be removed (since 1 == 1.0)
        assert!(tree.get("opacity").is_none());
    }

    #[test]
    fn test_opacity_string_not_touched() {
        let mut tree = json!({
            "name": "Test",
            "opacity": "1.0"
        });

        remove_default_opacity(&mut tree).unwrap();

        // String opacity should not be touched
        assert_eq!(tree.get("opacity").unwrap().as_str(), Some("1.0"));
    }
}
