use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove rotation field when it has the default value 0.0
///
/// Recursively traverses the JSON tree and removes "rotation" fields that have
/// the value 0.0. Since 0.0 is the default rotation (no rotation) in both Figma
/// and CSS, omitting it reduces output size without losing information.
///
/// This typically appears in image paint transforms and other transformation contexts.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all default rotation fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_default_rotation;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "image": {
///         "rotation": 0.0,
///         "scale": 0.5
///     }
/// });
/// remove_default_rotation(&mut tree).unwrap();
/// // image now has only "scale" field
/// ```
pub fn remove_default_rotation(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove default rotation fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check if rotation exists and is 0.0
            if let Some(rotation) = map.get("rotation") {
                if let Some(n) = rotation.as_f64() {
                    // Use epsilon comparison for floating point
                    if n.abs() < f64::EPSILON {
                        map.remove("rotation");
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
    fn test_remove_default_rotation() {
        let mut tree = json!({
            "name": "Image",
            "rotation": 0.0,
            "scale": 0.5
        });

        remove_default_rotation(&mut tree).unwrap();

        assert!(tree.get("rotation").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("scale").unwrap().as_f64(), Some(0.5));
    }

    #[test]
    fn test_preserve_non_zero_rotation() {
        let mut tree = json!({
            "name": "Image",
            "rotation": 45.0,
            "scale": 1.0
        });

        remove_default_rotation(&mut tree).unwrap();

        // Non-zero rotation should be preserved
        assert_eq!(tree.get("rotation").unwrap().as_f64(), Some(45.0));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
    }

    #[test]
    fn test_preserve_negative_rotation() {
        let mut tree = json!({
            "name": "Image",
            "rotation": -30.0
        });

        remove_default_rotation(&mut tree).unwrap();

        // Negative rotation should be preserved
        assert_eq!(tree.get("rotation").unwrap().as_f64(), Some(-30.0));
    }

    #[test]
    fn test_preserve_various_rotations() {
        let rotations = vec![15.0, 30.0, 45.0, 90.0, 180.0, 270.0, -45.0, -90.0];

        for rotation_value in rotations {
            let mut tree = json!({
                "rotation": rotation_value
            });

            remove_default_rotation(&mut tree).unwrap();

            // All non-zero rotations should be preserved
            assert_eq!(
                tree.get("rotation").unwrap().as_f64(),
                Some(rotation_value)
            );
        }
    }

    #[test]
    fn test_no_rotation() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_rotation(&mut tree).unwrap();

        // Tree without rotation should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("rotation").is_none());
    }

    #[test]
    fn test_rotation_in_image_paint() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "IMAGE",
                    "rotation": 0.0,
                    "scale": 0.5,
                    "image": {
                        "filename": "test.png"
                    }
                }
            ]
        });

        remove_default_rotation(&mut tree).unwrap();

        // rotation 0.0 should be removed
        assert!(tree["fillPaints"][0].get("rotation").is_none());
        assert_eq!(tree["fillPaints"][0]["scale"].as_f64(), Some(0.5));
    }

    #[test]
    fn test_rotation_in_transform() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "IMAGE",
                    "transform": {
                        "rotation": 0.0,
                        "x": 100.0,
                        "y": 200.0
                    }
                }
            ]
        });

        remove_default_rotation(&mut tree).unwrap();

        // rotation 0.0 should be removed from transform
        assert!(tree["fillPaints"][0]["transform"]
            .get("rotation")
            .is_none());
        assert_eq!(
            tree["fillPaints"][0]["transform"]["x"].as_f64(),
            Some(100.0)
        );
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "rotation": 0.0
                },
                {
                    "name": "Child2",
                    "rotation": 15.0
                }
            ]
        });

        remove_default_rotation(&mut tree).unwrap();

        // rotation 0.0 removed, 15.0 preserved
        assert!(tree["children"][0].get("rotation").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(tree["children"][1]["rotation"].as_f64(), Some(15.0));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "RECTANGLE",
                        "rotation": 0.0,
                        "fillPaints": [
                            {
                                "type": "IMAGE",
                                "rotation": 0.0
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_rotation(&mut tree).unwrap();

        // All rotation 0.0 should be removed at all levels
        let rect = &tree["document"]["children"][0];
        assert!(rect.get("rotation").is_none());
        assert!(rect["fillPaints"][0].get("rotation").is_none());
        assert_eq!(rect["type"].as_str(), Some("RECTANGLE"));
    }

    #[test]
    fn test_multiple_default_rotations() {
        let mut tree = json!({
            "children": [
                {"rotation": 0.0, "name": "A"},
                {"rotation": 0.0, "name": "B"},
                {"rotation": 0.0, "name": "C"}
            ]
        });

        remove_default_rotation(&mut tree).unwrap();

        // All rotation 0.0 should be removed
        assert!(tree["children"][0].get("rotation").is_none());
        assert!(tree["children"][1].get("rotation").is_none());
        assert!(tree["children"][2].get("rotation").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_rotation_as_integer() {
        let mut tree = json!({
            "name": "Shape",
            "rotation": 0
        });

        remove_default_rotation(&mut tree).unwrap();

        // Integer 0 should also be removed (since 0 == 0.0)
        assert!(tree.get("rotation").is_none());
    }

    #[test]
    fn test_rotation_string_not_touched() {
        let mut tree = json!({
            "name": "Test",
            "rotation": "0.0"
        });

        remove_default_rotation(&mut tree).unwrap();

        // String rotation should not be touched
        assert_eq!(tree.get("rotation").unwrap().as_str(), Some("0.0"));
    }
}
