use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove blendMode field when it has the default value "NORMAL"
///
/// Recursively traverses the JSON tree and removes "blendMode" fields that have
/// the value "NORMAL" (after enum simplification has converted them from enum
/// objects to strings). NORMAL is the default blend mode in both Figma and CSS,
/// so omitting it reduces output size without losing information.
///
/// IMPORTANT: This transformation must run AFTER enum_simplification, which
/// converts `{"__enum__": "BlendMode", "value": "NORMAL"}` to `"NORMAL"`.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all default blendMode fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_default_blend_mode;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "blendMode": "NORMAL",
///     "opacity": 1.0
/// });
/// remove_default_blend_mode(&mut tree).unwrap();
/// // tree now has only "name" and "opacity" fields
/// ```
pub fn remove_default_blend_mode(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove default blendMode fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check if blendMode exists and is "NORMAL"
            if let Some(blend_mode) = map.get("blendMode") {
                if let Some(s) = blend_mode.as_str() {
                    if s == "NORMAL" {
                        map.remove("blendMode");
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
    fn test_remove_normal_blend_mode() {
        let mut tree = json!({
            "name": "Shape",
            "blendMode": "NORMAL",
            "opacity": 1.0
        });

        remove_default_blend_mode(&mut tree).unwrap();

        assert!(tree.get("blendMode").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_preserve_non_normal_blend_mode() {
        let mut tree = json!({
            "name": "Shape",
            "blendMode": "MULTIPLY",
            "opacity": 0.8
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // Non-NORMAL blend modes should be preserved
        assert_eq!(tree.get("blendMode").unwrap().as_str(), Some("MULTIPLY"));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.8));
    }

    #[test]
    fn test_preserve_other_blend_modes() {
        let modes = vec!["MULTIPLY", "SCREEN", "OVERLAY", "DARKEN", "LIGHTEN"];

        for mode in modes {
            let mut tree = json!({
                "blendMode": mode
            });

            remove_default_blend_mode(&mut tree).unwrap();

            // All non-NORMAL blend modes should be preserved
            assert_eq!(tree.get("blendMode").unwrap().as_str(), Some(mode));
        }
    }

    #[test]
    fn test_no_blend_mode() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // Tree without blendMode should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("blendMode").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "blendMode": "NORMAL"
                },
                {
                    "name": "Child2",
                    "blendMode": "MULTIPLY"
                }
            ]
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // NORMAL removed, MULTIPLY preserved
        assert!(tree["children"][0].get("blendMode").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(
            tree["children"][1]["blendMode"].as_str(),
            Some("MULTIPLY")
        );
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_blend_mode_in_paints() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "SOLID",
                    "blendMode": "NORMAL",
                    "color": "#ff0000"
                },
                {
                    "type": "GRADIENT",
                    "blendMode": "MULTIPLY",
                    "color": "#00ff00"
                }
            ]
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // NORMAL removed from first paint
        assert!(tree["fillPaints"][0].get("blendMode").is_none());
        assert_eq!(tree["fillPaints"][0]["type"].as_str(), Some("SOLID"));

        // MULTIPLY preserved in second paint
        assert_eq!(
            tree["fillPaints"][1]["blendMode"].as_str(),
            Some("MULTIPLY")
        );
        assert_eq!(tree["fillPaints"][1]["type"].as_str(), Some("GRADIENT"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "blendMode": "NORMAL",
                        "fillPaints": [
                            {
                                "type": "SOLID",
                                "blendMode": "NORMAL"
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // All NORMAL blend modes should be removed at all levels
        let frame = &tree["document"]["children"][0];
        assert!(frame.get("blendMode").is_none());
        assert!(frame["fillPaints"][0].get("blendMode").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_blend_mode_enum_object_not_touched() {
        let mut tree = json!({
            "name": "Shape",
            "blendMode": {
                "__enum__": "BlendMode",
                "value": "NORMAL"
            }
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // Enum objects should not be touched (this runs after enum_simplification)
        // So this should be preserved as-is
        assert!(tree.get("blendMode").is_some());
        let blend_mode = tree.get("blendMode").unwrap();
        assert!(blend_mode.is_object());
    }

    #[test]
    fn test_case_sensitive() {
        let mut tree = json!({
            "blendMode": "normal"
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // Lowercase "normal" should not be removed (only "NORMAL")
        assert_eq!(tree.get("blendMode").unwrap().as_str(), Some("normal"));
    }

    #[test]
    fn test_multiple_normal_blend_modes() {
        let mut tree = json!({
            "children": [
                {"blendMode": "NORMAL", "name": "A"},
                {"blendMode": "NORMAL", "name": "B"},
                {"blendMode": "NORMAL", "name": "C"}
            ]
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // All NORMAL blend modes should be removed
        assert!(tree["children"][0].get("blendMode").is_none());
        assert!(tree["children"][1].get("blendMode").is_none());
        assert!(tree["children"][2].get("blendMode").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }
}
