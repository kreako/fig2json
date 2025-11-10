use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove uniformScaleFactor field when it has the default value 1.0
///
/// Recursively traverses the JSON tree and removes "uniformScaleFactor" fields that have
/// the value 1.0. This is the default scale factor, so omitting it reduces output size
/// without losing information.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all default uniformScaleFactor fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_default_uniform_scale_factor;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "uniformScaleFactor": 1.0,
///     "width": 100
/// });
/// remove_default_uniform_scale_factor(&mut tree).unwrap();
/// // tree now has only "name" and "width" fields
/// ```
pub fn remove_default_uniform_scale_factor(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove default uniformScaleFactor fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check if uniformScaleFactor exists and is 1.0
            if let Some(scale_factor) = map.get("uniformScaleFactor") {
                if let Some(num) = scale_factor.as_f64() {
                    // Remove if exactly 1.0
                    if (num - 1.0).abs() < f64::EPSILON {
                        map.remove("uniformScaleFactor");
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
    fn test_remove_default_uniform_scale_factor() {
        let mut tree = json!({
            "name": "Shape",
            "uniformScaleFactor": 1.0,
            "width": 100
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        assert!(tree.get("uniformScaleFactor").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
    }

    #[test]
    fn test_preserve_non_default_scale_factor() {
        let mut tree = json!({
            "name": "Shape",
            "uniformScaleFactor": 2.5,
            "width": 100
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // Non-default scale factors should be preserved
        assert_eq!(
            tree.get("uniformScaleFactor").unwrap().as_f64(),
            Some(2.5)
        );
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
    }

    #[test]
    fn test_preserve_various_scale_factors() {
        let scale_factors = vec![0.5, 1.5, 2.0, 0.1, 10.0];

        for factor in scale_factors {
            let mut tree = json!({
                "uniformScaleFactor": factor
            });

            remove_default_uniform_scale_factor(&mut tree).unwrap();

            // All non-default scale factors should be preserved
            assert_eq!(
                tree.get("uniformScaleFactor").unwrap().as_f64(),
                Some(factor)
            );
        }
    }

    #[test]
    fn test_no_scale_factor() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // Tree without uniformScaleFactor should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("uniformScaleFactor").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "uniformScaleFactor": 1.0
                },
                {
                    "name": "Child2",
                    "uniformScaleFactor": 2.0
                }
            ]
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // Default (1.0) removed, 2.0 preserved
        assert!(tree["children"][0].get("uniformScaleFactor").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(
            tree["children"][1]["uniformScaleFactor"].as_f64(),
            Some(2.0)
        );
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "uniformScaleFactor": 1.0,
                        "layers": [
                            {
                                "type": "SHAPE",
                                "uniformScaleFactor": 1.0
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // All default scale factors should be removed at all levels
        let frame = &tree["document"]["children"][0];
        assert!(frame.get("uniformScaleFactor").is_none());
        assert!(frame["layers"][0].get("uniformScaleFactor").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_multiple_default_scale_factors() {
        let mut tree = json!({
            "children": [
                {"uniformScaleFactor": 1.0, "name": "A"},
                {"uniformScaleFactor": 1.0, "name": "B"},
                {"uniformScaleFactor": 1.0, "name": "C"}
            ]
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // All default scale factors should be removed
        assert!(tree["children"][0].get("uniformScaleFactor").is_none());
        assert!(tree["children"][1].get("uniformScaleFactor").is_none());
        assert!(tree["children"][2].get("uniformScaleFactor").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_zero_scale_factor() {
        let mut tree = json!({
            "uniformScaleFactor": 0.0
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // Zero is not the default, so it should be preserved
        assert_eq!(tree.get("uniformScaleFactor").unwrap().as_f64(), Some(0.0));
    }

    #[test]
    fn test_integer_one_vs_float_one() {
        // JSON doesn't distinguish between 1 and 1.0, both are treated as numbers
        let mut tree = json!({
            "uniformScaleFactor": 1
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // Integer 1 should also be removed (it's the same as 1.0)
        assert!(tree.get("uniformScaleFactor").is_none());
    }
}
