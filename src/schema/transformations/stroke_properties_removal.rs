use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove stroke property fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes stroke-related fields:
/// - "strokeAlign" - Stroke alignment (INSIDE/CENTER/OUTSIDE) not supported in CSS
/// - "strokeJoin" - Stroke join style (MITER/BEVEL/ROUND)
/// - "strokeWeight" - Stroke width value
///
/// These fields contain stroke properties that don't have direct CSS equivalents
/// or are not essential for basic HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all stroke property fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_stroke_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "strokeAlign": {
///         "__enum__": "StrokeAlign",
///         "value": "INSIDE"
///     },
///     "strokeJoin": {
///         "__enum__": "StrokeJoin",
///         "value": "MITER"
///     },
///     "strokeWeight": 1.0,
///     "visible": true
/// });
/// remove_stroke_properties(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_stroke_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove stroke property fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove stroke property fields if they exist
            map.remove("strokeAlign");
            map.remove("strokeJoin");
            map.remove("strokeWeight");

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
    fn test_remove_stroke_align() {
        let mut tree = json!({
            "name": "Rectangle",
            "strokeAlign": {
                "__enum__": "StrokeAlign",
                "value": "INSIDE"
            },
            "visible": true
        });

        remove_stroke_properties(&mut tree).unwrap();

        assert!(tree.get("strokeAlign").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_stroke_join() {
        let mut tree = json!({
            "name": "Line",
            "strokeJoin": {
                "__enum__": "StrokeJoin",
                "value": "MITER"
            },
            "opacity": 1.0
        });

        remove_stroke_properties(&mut tree).unwrap();

        assert!(tree.get("strokeJoin").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Line"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_remove_stroke_weight() {
        let mut tree = json!({
            "name": "Shape",
            "strokeWeight": 1.0,
            "visible": true
        });

        remove_stroke_properties(&mut tree).unwrap();

        assert!(tree.get("strokeWeight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_all_stroke_properties() {
        let mut tree = json!({
            "name": "Rectangle",
            "strokeAlign": {
                "__enum__": "StrokeAlign",
                "value": "CENTER"
            },
            "strokeJoin": {
                "__enum__": "StrokeJoin",
                "value": "BEVEL"
            },
            "strokeWeight": 2.5,
            "opacity": 1.0
        });

        remove_stroke_properties(&mut tree).unwrap();

        assert!(tree.get("strokeAlign").is_none());
        assert!(tree.get("strokeJoin").is_none());
        assert!(tree.get("strokeWeight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_nested_stroke_properties() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "strokeAlign": {
                        "__enum__": "StrokeAlign",
                        "value": "INSIDE"
                    },
                    "strokeWeight": 1.0
                },
                {
                    "name": "Child2",
                    "children": [
                        {
                            "name": "DeepChild",
                            "strokeJoin": {
                                "__enum__": "StrokeJoin",
                                "value": "ROUND"
                            }
                        }
                    ]
                }
            ]
        });

        remove_stroke_properties(&mut tree).unwrap();

        // Check first nested element
        assert!(tree["children"][0].get("strokeAlign").is_none());
        assert!(tree["children"][0].get("strokeWeight").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child1")
        );

        // Check deeply nested element
        let deep_child = &tree["children"][1]["children"][0];
        assert!(deep_child.get("strokeJoin").is_none());
        assert_eq!(deep_child.get("name").unwrap().as_str(), Some("DeepChild"));
    }

    #[test]
    fn test_no_stroke_properties() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "visible": true
        });

        remove_stroke_properties(&mut tree).unwrap();

        // Tree without stroke properties should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_preserves_stroke_paints() {
        let mut tree = json!({
            "name": "Line",
            "strokeAlign": {
                "__enum__": "StrokeAlign",
                "value": "CENTER"
            },
            "strokeWeight": 1.0,
            "strokePaints": [
                {
                    "blendMode": {
                        "__enum__": "BlendMode",
                        "value": "NORMAL"
                    },
                    "color": "#000000",
                    "opacity": 1.0,
                    "visible": true
                }
            ]
        });

        remove_stroke_properties(&mut tree).unwrap();

        // Stroke properties removed
        assert!(tree.get("strokeAlign").is_none());
        assert!(tree.get("strokeWeight").is_none());

        // strokePaints preserved (contains actual stroke color data)
        assert!(tree.get("strokePaints").is_some());
        assert_eq!(tree["strokePaints"][0]["color"].as_str(), Some("#000000"));
    }

    #[test]
    fn test_multiple_objects_with_stroke_properties() {
        let mut tree = json!({
            "items": [
                {
                    "name": "Item1",
                    "strokeAlign": {
                        "__enum__": "StrokeAlign",
                        "value": "INSIDE"
                    }
                },
                {
                    "name": "Item2",
                    "strokeJoin": {
                        "__enum__": "StrokeJoin",
                        "value": "MITER"
                    }
                },
                {
                    "name": "Item3",
                    "strokeWeight": 3.0
                }
            ]
        });

        remove_stroke_properties(&mut tree).unwrap();

        // All stroke properties in array should be removed
        assert!(tree["items"][0].get("strokeAlign").is_none());
        assert_eq!(
            tree["items"][0].get("name").unwrap().as_str(),
            Some("Item1")
        );

        assert!(tree["items"][1].get("strokeJoin").is_none());
        assert_eq!(
            tree["items"][1].get("name").unwrap().as_str(),
            Some("Item2")
        );

        assert!(tree["items"][2].get("strokeWeight").is_none());
        assert_eq!(
            tree["items"][2].get("name").unwrap().as_str(),
            Some("Item3")
        );
    }

    #[test]
    fn test_different_stroke_align_values() {
        let mut tree = json!({
            "shape1": {
                "strokeAlign": {
                    "__enum__": "StrokeAlign",
                    "value": "INSIDE"
                }
            },
            "shape2": {
                "strokeAlign": {
                    "__enum__": "StrokeAlign",
                    "value": "CENTER"
                }
            },
            "shape3": {
                "strokeAlign": {
                    "__enum__": "StrokeAlign",
                    "value": "OUTSIDE"
                }
            }
        });

        remove_stroke_properties(&mut tree).unwrap();

        // All variations of strokeAlign should be removed
        assert!(tree["shape1"].get("strokeAlign").is_none());
        assert!(tree["shape2"].get("strokeAlign").is_none());
        assert!(tree["shape3"].get("strokeAlign").is_none());
    }
}
