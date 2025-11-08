use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove individual border weight fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes Figma-specific border weight fields:
/// - "borderTopWeight" - Top border weight
/// - "borderBottomWeight" - Bottom border weight
/// - "borderLeftWeight" - Left border weight
/// - "borderRightWeight" - Right border weight
/// - "borderStrokeWeightsIndependent" - Flag indicating independent border weights
///
/// These fields allow per-side border weights in Figma, but standard HTML/CSS
/// uses uniform borders. For HTML/CSS rendering, these detailed border weights
/// are not needed and can be safely removed.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all border weight fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_border_weights;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "borderTopWeight": 1.0,
///     "borderBottomWeight": 1.0,
///     "borderLeftWeight": 1.0,
///     "borderRightWeight": 1.0,
///     "borderStrokeWeightsIndependent": true,
///     "visible": true
/// });
/// remove_border_weights(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_border_weights(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove border weight fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove all border weight fields if they exist
            map.remove("borderTopWeight");
            map.remove("borderBottomWeight");
            map.remove("borderLeftWeight");
            map.remove("borderRightWeight");
            map.remove("borderStrokeWeightsIndependent");

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
    fn test_remove_all_border_weights() {
        let mut tree = json!({
            "name": "Rectangle",
            "borderTopWeight": 1.0,
            "borderBottomWeight": 2.0,
            "borderLeftWeight": 1.5,
            "borderRightWeight": 2.5,
            "visible": true
        });

        remove_border_weights(&mut tree).unwrap();

        assert!(tree.get("borderTopWeight").is_none());
        assert!(tree.get("borderBottomWeight").is_none());
        assert!(tree.get("borderLeftWeight").is_none());
        assert!(tree.get("borderRightWeight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_partial_border_weights() {
        let mut tree = json!({
            "name": "Shape",
            "borderTopWeight": 1.0,
            "borderLeftWeight": 1.0,
            "width": 100
        });

        remove_border_weights(&mut tree).unwrap();

        assert!(tree.get("borderTopWeight").is_none());
        assert!(tree.get("borderLeftWeight").is_none());
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
    }

    #[test]
    fn test_no_border_weights() {
        let mut tree = json!({
            "name": "Circle",
            "radius": 50,
            "visible": true
        });

        remove_border_weights(&mut tree).unwrap();

        // Tree without border weights should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Circle"));
        assert_eq!(tree.get("radius").unwrap().as_i64(), Some(50));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "borderTopWeight": 1.0,
                    "borderBottomWeight": 1.0
                },
                {
                    "name": "Child2",
                    "borderLeftWeight": 2.0,
                    "borderRightWeight": 2.0
                }
            ]
        });

        remove_border_weights(&mut tree).unwrap();

        // All nested border weights should be removed
        assert!(tree["children"][0].get("borderTopWeight").is_none());
        assert!(tree["children"][0].get("borderBottomWeight").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert!(tree["children"][1].get("borderLeftWeight").is_none());
        assert!(tree["children"][1].get("borderRightWeight").is_none());
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "borderTopWeight": 1.0,
                        "borderBottomWeight": 1.0,
                        "borderLeftWeight": 1.0,
                        "borderRightWeight": 1.0
                    }
                ]
            }
        });

        remove_border_weights(&mut tree).unwrap();

        let frame = &tree["document"]["children"][0];
        assert!(frame.get("borderTopWeight").is_none());
        assert!(frame.get("borderBottomWeight").is_none());
        assert!(frame.get("borderLeftWeight").is_none());
        assert!(frame.get("borderRightWeight").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_preserve_other_border_properties() {
        let mut tree = json!({
            "name": "Rectangle",
            "borderTopWeight": 1.0,
            "borderRadius": 5.0,
            "borderColor": "#ff0000"
        });

        remove_border_weights(&mut tree).unwrap();

        // Other border properties should be preserved
        assert!(tree.get("borderTopWeight").is_none());
        assert_eq!(tree.get("borderRadius").unwrap().as_f64(), Some(5.0));
        assert_eq!(tree.get("borderColor").unwrap().as_str(), Some("#ff0000"));
    }

    #[test]
    fn test_multiple_shapes_in_array() {
        let mut tree = json!({
            "shapes": [
                {
                    "type": "rectangle",
                    "borderTopWeight": 1.0,
                    "borderBottomWeight": 1.0
                },
                {
                    "type": "ellipse",
                    "borderLeftWeight": 2.0,
                    "borderRightWeight": 2.0
                },
                {
                    "type": "line",
                    "borderTopWeight": 0.5
                }
            ]
        });

        remove_border_weights(&mut tree).unwrap();

        // All border weights in all array elements should be removed
        assert!(tree["shapes"][0].get("borderTopWeight").is_none());
        assert!(tree["shapes"][0].get("borderBottomWeight").is_none());
        assert_eq!(tree["shapes"][0]["type"].as_str(), Some("rectangle"));

        assert!(tree["shapes"][1].get("borderLeftWeight").is_none());
        assert!(tree["shapes"][1].get("borderRightWeight").is_none());
        assert_eq!(tree["shapes"][1]["type"].as_str(), Some("ellipse"));

        assert!(tree["shapes"][2].get("borderTopWeight").is_none());
        assert_eq!(tree["shapes"][2]["type"].as_str(), Some("line"));
    }

    #[test]
    fn test_zero_border_weights() {
        let mut tree = json!({
            "name": "Shape",
            "borderTopWeight": 0.0,
            "borderBottomWeight": 0.0,
            "borderLeftWeight": 0.0,
            "borderRightWeight": 0.0
        });

        remove_border_weights(&mut tree).unwrap();

        // Even zero-value border weights should be removed
        assert!(tree.get("borderTopWeight").is_none());
        assert!(tree.get("borderBottomWeight").is_none());
        assert!(tree.get("borderLeftWeight").is_none());
        assert!(tree.get("borderRightWeight").is_none());
    }

    #[test]
    fn test_remove_border_stroke_weights_independent() {
        let mut tree = json!({
            "name": "Rectangle",
            "borderStrokeWeightsIndependent": true,
            "borderTopWeight": 1.0,
            "borderBottomWeight": 2.0,
            "visible": true
        });

        remove_border_weights(&mut tree).unwrap();

        assert!(tree.get("borderStrokeWeightsIndependent").is_none());
        assert!(tree.get("borderTopWeight").is_none());
        assert!(tree.get("borderBottomWeight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_border_stroke_weights_independent_nested() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "borderStrokeWeightsIndependent": true,
                    "borderTopWeight": 1.0
                },
                {
                    "name": "Child2",
                    "borderStrokeWeightsIndependent": false
                }
            ]
        });

        remove_border_weights(&mut tree).unwrap();

        assert!(tree["children"][0].get("borderStrokeWeightsIndependent").is_none());
        assert!(tree["children"][0].get("borderTopWeight").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert!(tree["children"][1].get("borderStrokeWeightsIndependent").is_none());
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }
}
