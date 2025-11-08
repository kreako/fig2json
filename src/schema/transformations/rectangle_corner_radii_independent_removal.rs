use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove rectangleCornerRadiiIndependent fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes all "rectangleCornerRadiiIndependent" fields.
/// This flag indicates whether corner radii are set independently, which is redundant when
/// the actual corner radius values are present.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all rectangleCornerRadiiIndependent fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_rectangle_corner_radii_independent;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "cornerRadius": 16.0,
///     "rectangleCornerRadiiIndependent": true,
///     "rectangleTopLeftCornerRadius": 16.0,
///     "rectangleTopRightCornerRadius": 16.0,
///     "visible": true
/// });
/// remove_rectangle_corner_radii_independent(&mut tree).unwrap();
/// // tree now has cornerRadius and specific radius fields but not the flag
/// ```
pub fn remove_rectangle_corner_radii_independent(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove rectangleCornerRadiiIndependent fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the "rectangleCornerRadiiIndependent" field if it exists
            map.remove("rectangleCornerRadiiIndependent");

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
    fn test_remove_rectangle_corner_radii_independent_simple() {
        let mut tree = json!({
            "name": "Rectangle",
            "cornerRadius": 16.0,
            "rectangleCornerRadiiIndependent": true,
            "visible": true
        });

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert!(tree.get("rectangleCornerRadiiIndependent").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("cornerRadius").unwrap().as_f64(), Some(16.0));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_rectangle_corner_radii_independent_with_specific_radii() {
        let mut tree = json!({
            "name": "RoundedRect",
            "cornerRadius": 16.0,
            "rectangleCornerRadiiIndependent": true,
            "rectangleTopLeftCornerRadius": 16.0,
            "rectangleTopRightCornerRadius": 16.0,
            "rectangleBottomLeftCornerRadius": 16.0,
            "rectangleBottomRightCornerRadius": 16.0
        });

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert!(tree.get("rectangleCornerRadiiIndependent").is_none());
        assert_eq!(tree.get("cornerRadius").unwrap().as_f64(), Some(16.0));
        assert_eq!(
            tree.get("rectangleTopLeftCornerRadius").unwrap().as_f64(),
            Some(16.0)
        );
        assert_eq!(
            tree.get("rectangleTopRightCornerRadius").unwrap().as_f64(),
            Some(16.0)
        );
    }

    #[test]
    fn test_remove_rectangle_corner_radii_independent_false() {
        let mut tree = json!({
            "name": "Rectangle",
            "rectangleCornerRadiiIndependent": false,
            "cornerRadius": 8.0
        });

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert!(tree.get("rectangleCornerRadiiIndependent").is_none());
        assert_eq!(tree.get("cornerRadius").unwrap().as_f64(), Some(8.0));
    }

    #[test]
    fn test_remove_rectangle_corner_radii_independent_nested() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Rect1",
                    "rectangleCornerRadiiIndependent": true,
                    "cornerRadius": 12.0
                },
                {
                    "name": "Rect2",
                    "rectangleCornerRadiiIndependent": false,
                    "cornerRadius": 8.0
                }
            ]
        });

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert!(tree["children"][0]
            .get("rectangleCornerRadiiIndependent")
            .is_none());
        assert!(tree["children"][1]
            .get("rectangleCornerRadiiIndependent")
            .is_none());
        assert_eq!(tree["children"][0]["cornerRadius"].as_f64(), Some(12.0));
        assert_eq!(tree["children"][1]["cornerRadius"].as_f64(), Some(8.0));
    }

    #[test]
    fn test_remove_rectangle_corner_radii_independent_deeply_nested() {
        let mut tree = json!({
            "document": {
                "rectangleCornerRadiiIndependent": true,
                "children": [
                    {
                        "rectangleCornerRadiiIndependent": true,
                        "children": [
                            {
                                "rectangleCornerRadiiIndependent": false,
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert!(tree["document"]
            .get("rectangleCornerRadiiIndependent")
            .is_none());
        assert!(tree["document"]["children"][0]
            .get("rectangleCornerRadiiIndependent")
            .is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("rectangleCornerRadiiIndependent")
            .is_none());
    }

    #[test]
    fn test_remove_rectangle_corner_radii_independent_missing() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "visible": true
        });

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert!(tree.get("rectangleCornerRadiiIndependent").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_remove_rectangle_corner_radii_independent_preserves_other_fields() {
        let mut tree = json!({
            "name": "Row",
            "cornerRadius": 16.0,
            "rectangleCornerRadiiIndependent": true,
            "rectangleTopLeftCornerRadius": 16.0,
            "rectangleTopRightCornerRadius": 16.0,
            "rectangleBottomLeftCornerRadius": 16.0,
            "rectangleBottomRightCornerRadius": 16.0,
            "fillPaints": [{"color": "#18181b", "type": "SOLID"}],
            "size": {"x": 327.0, "y": 64.0},
            "visible": true
        });

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert!(tree.get("rectangleCornerRadiiIndependent").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Row"));
        assert_eq!(tree.get("cornerRadius").unwrap().as_f64(), Some(16.0));
        assert!(tree.get("fillPaints").is_some());
        assert!(tree.get("size").is_some());
        // Specific radii should be preserved
        assert!(tree.get("rectangleTopLeftCornerRadius").is_some());
        assert!(tree.get("rectangleTopRightCornerRadius").is_some());
    }

    #[test]
    fn test_remove_rectangle_corner_radii_independent_in_arrays() {
        let mut tree = json!({
            "rectangles": [
                {
                    "name": "Rect1",
                    "rectangleCornerRadiiIndependent": true,
                    "cornerRadius": 10.0
                },
                {
                    "name": "Rect2",
                    "rectangleCornerRadiiIndependent": false,
                    "cornerRadius": 5.0
                }
            ]
        });

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert!(tree["rectangles"][0]
            .get("rectangleCornerRadiiIndependent")
            .is_none());
        assert_eq!(
            tree["rectangles"][0].get("name").unwrap().as_str(),
            Some("Rect1")
        );
        assert!(tree["rectangles"][1]
            .get("rectangleCornerRadiiIndependent")
            .is_none());
        assert_eq!(
            tree["rectangles"][1].get("name").unwrap().as_str(),
            Some("Rect2")
        );
    }

    #[test]
    fn test_remove_rectangle_corner_radii_independent_empty_object() {
        let mut tree = json!({});

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_remove_rectangle_corner_radii_independent_primitives() {
        let mut tree = json!(42);

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert_eq!(tree.as_i64(), Some(42));
    }

    #[test]
    fn test_remove_rectangle_corner_radii_independent_mixed_nodes() {
        let mut tree = json!({
            "children": [
                {
                    "name": "WithFlag",
                    "rectangleCornerRadiiIndependent": true
                },
                {
                    "name": "WithoutFlag"
                },
                {
                    "name": "AlsoWithFlag",
                    "rectangleCornerRadiiIndependent": false
                }
            ]
        });

        remove_rectangle_corner_radii_independent(&mut tree).unwrap();

        assert!(tree["children"][0]
            .get("rectangleCornerRadiiIndependent")
            .is_none());
        assert!(tree["children"][1]
            .get("rectangleCornerRadiiIndependent")
            .is_none());
        assert!(tree["children"][2]
            .get("rectangleCornerRadiiIndependent")
            .is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("WithFlag"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("WithoutFlag"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("AlsoWithFlag"));
    }
}
