use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove frame property fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes frame-specific fields:
/// - "frameMaskDisabled" - Frame mask disabled flag
/// - "targetAspectRatio" - Target aspect ratio for frame
///
/// These fields contain frame-specific configuration that is not needed for
/// basic HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all frame property fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_frame_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "frameMaskDisabled": false,
///     "targetAspectRatio": {
///         "value": {
///             "x": 300.0,
///             "y": 300.0
///         }
///     },
///     "visible": true
/// });
/// remove_frame_properties(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_frame_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove frame property fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove frame property fields if they exist
            map.remove("frameMaskDisabled");
            map.remove("targetAspectRatio");

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
    fn test_remove_frame_mask_disabled() {
        let mut tree = json!({
            "name": "Frame",
            "frameMaskDisabled": false,
            "visible": true
        });

        remove_frame_properties(&mut tree).unwrap();

        assert!(tree.get("frameMaskDisabled").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_target_aspect_ratio() {
        let mut tree = json!({
            "name": "Image",
            "targetAspectRatio": {
                "value": {
                    "x": 300.0,
                    "y": 300.0
                }
            },
            "opacity": 1.0
        });

        remove_frame_properties(&mut tree).unwrap();

        assert!(tree.get("targetAspectRatio").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_remove_both_frame_properties() {
        let mut tree = json!({
            "name": "Frame",
            "frameMaskDisabled": false,
            "targetAspectRatio": {
                "value": {
                    "x": 16.0,
                    "y": 9.0
                }
            },
            "type": "FRAME"
        });

        remove_frame_properties(&mut tree).unwrap();

        assert!(tree.get("frameMaskDisabled").is_none());
        assert!(tree.get("targetAspectRatio").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
    }

    #[test]
    fn test_nested_frame_properties() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "frameMaskDisabled": false
                },
                {
                    "name": "Child2",
                    "children": [
                        {
                            "name": "DeepChild",
                            "targetAspectRatio": {
                                "value": {
                                    "x": 100.0,
                                    "y": 100.0
                                }
                            }
                        }
                    ]
                }
            ]
        });

        remove_frame_properties(&mut tree).unwrap();

        // Check first nested element
        assert!(tree["children"][0].get("frameMaskDisabled").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child1")
        );

        // Check deeply nested element
        let deep_child = &tree["children"][1]["children"][0];
        assert!(deep_child.get("targetAspectRatio").is_none());
        assert_eq!(deep_child.get("name").unwrap().as_str(), Some("DeepChild"));
    }

    #[test]
    fn test_no_frame_properties() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "visible": true
        });

        remove_frame_properties(&mut tree).unwrap();

        // Tree without frame properties should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_preserves_other_frame_fields() {
        let mut tree = json!({
            "name": "Ecran 1",
            "frameMaskDisabled": false,
            "type": "FRAME",
            "size": {
                "x": 1170.0,
                "y": 2532.0
            },
            "fillPaints": [
                {
                    "color": "#ffffff",
                    "opacity": 1.0,
                    "visible": true
                }
            ]
        });

        remove_frame_properties(&mut tree).unwrap();

        // Frame properties removed
        assert!(tree.get("frameMaskDisabled").is_none());

        // Other fields preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Ecran 1"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert!(tree.get("size").is_some());
        assert!(tree.get("fillPaints").is_some());
    }

    #[test]
    fn test_multiple_frames() {
        let mut tree = json!({
            "items": [
                {
                    "name": "Frame1",
                    "frameMaskDisabled": false
                },
                {
                    "name": "Frame2",
                    "frameMaskDisabled": true
                },
                {
                    "name": "Image1",
                    "targetAspectRatio": {
                        "value": {
                            "x": 4.0,
                            "y": 3.0
                        }
                    }
                }
            ]
        });

        remove_frame_properties(&mut tree).unwrap();

        // All frame properties in array should be removed
        assert!(tree["items"][0].get("frameMaskDisabled").is_none());
        assert_eq!(
            tree["items"][0].get("name").unwrap().as_str(),
            Some("Frame1")
        );

        assert!(tree["items"][1].get("frameMaskDisabled").is_none());
        assert_eq!(
            tree["items"][1].get("name").unwrap().as_str(),
            Some("Frame2")
        );

        assert!(tree["items"][2].get("targetAspectRatio").is_none());
        assert_eq!(
            tree["items"][2].get("name").unwrap().as_str(),
            Some("Image1")
        );
    }

    #[test]
    fn test_frame_mask_disabled_different_values() {
        let mut tree = json!({
            "frame1": {
                "frameMaskDisabled": false,
                "name": "Frame1"
            },
            "frame2": {
                "frameMaskDisabled": true,
                "name": "Frame2"
            }
        });

        remove_frame_properties(&mut tree).unwrap();

        // Both true and false values should be removed
        assert!(tree["frame1"].get("frameMaskDisabled").is_none());
        assert_eq!(tree["frame1"].get("name").unwrap().as_str(), Some("Frame1"));

        assert!(tree["frame2"].get("frameMaskDisabled").is_none());
        assert_eq!(tree["frame2"].get("name").unwrap().as_str(), Some("Frame2"));
    }

    #[test]
    fn test_empty_object() {
        let mut tree = json!({});

        remove_frame_properties(&mut tree).unwrap();

        // Empty object should remain empty
        assert_eq!(tree.as_object().unwrap().len(), 0);
    }
}
