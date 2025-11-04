use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove default text property values from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes text properties that have
/// default values to reduce JSON size:
/// - "letterSpacing" with value 0 and units "PERCENT" (default)
/// - "lineHeight" with value 100 and units "PERCENT" (default, equivalent to 1.0)
///
/// These are the default values in Figma and CSS, so omitting them reduces
/// output size without losing information.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all default text property fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_default_text_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Text",
///     "letterSpacing": {"units": "PERCENT", "value": 0.0},
///     "lineHeight": {"units": "PERCENT", "value": 100.0},
///     "fontSize": 16.0
/// });
/// remove_default_text_properties(&mut tree).unwrap();
/// // tree now has only "name" and "fontSize" fields
/// ```
pub fn remove_default_text_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove default text properties from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check and remove letterSpacing if it's default (0 PERCENT)
            if let Some(letter_spacing) = map.get("letterSpacing") {
                if is_default_letter_spacing(letter_spacing) {
                    map.remove("letterSpacing");
                }
            }

            // Check and remove lineHeight if it's default (100 PERCENT)
            if let Some(line_height) = map.get("lineHeight") {
                if is_default_line_height(line_height) {
                    map.remove("lineHeight");
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

/// Check if letterSpacing has default value (0 PERCENT)
fn is_default_letter_spacing(value: &JsonValue) -> bool {
    if let Some(obj) = value.as_object() {
        let has_percent_units = obj
            .get("units")
            .and_then(|v| v.as_str())
            .map(|s| s == "PERCENT")
            .unwrap_or(false);

        let has_zero_value = obj
            .get("value")
            .and_then(|v| v.as_f64())
            .map(|f| f.abs() < 1e-10)
            .unwrap_or(false);

        has_percent_units && has_zero_value
    } else {
        false
    }
}

/// Check if lineHeight has default value (100 PERCENT)
fn is_default_line_height(value: &JsonValue) -> bool {
    if let Some(obj) = value.as_object() {
        let has_percent_units = obj
            .get("units")
            .and_then(|v| v.as_str())
            .map(|s| s == "PERCENT")
            .unwrap_or(false);

        let has_hundred_value = obj
            .get("value")
            .and_then(|v| v.as_f64())
            .map(|f| (f - 100.0).abs() < 1e-10)
            .unwrap_or(false);

        has_percent_units && has_hundred_value
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_default_letter_spacing() {
        let mut tree = json!({
            "name": "Text",
            "letterSpacing": {"units": "PERCENT", "value": 0.0},
            "fontSize": 16.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        assert!(tree.get("letterSpacing").is_none());
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_remove_default_line_height() {
        let mut tree = json!({
            "name": "Text",
            "lineHeight": {"units": "PERCENT", "value": 100.0},
            "fontSize": 16.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        assert!(tree.get("lineHeight").is_none());
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_remove_both_defaults() {
        let mut tree = json!({
            "name": "Text",
            "letterSpacing": {"units": "PERCENT", "value": 0.0},
            "lineHeight": {"units": "PERCENT", "value": 100.0},
            "fontSize": 14.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        assert!(tree.get("letterSpacing").is_none());
        assert!(tree.get("lineHeight").is_none());
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_preserve_non_default_letter_spacing() {
        let mut tree = json!({
            "name": "Text",
            "letterSpacing": {"units": "PERCENT", "value": 5.0},
            "fontSize": 16.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        // Non-default letterSpacing should be preserved
        assert!(tree.get("letterSpacing").is_some());
        assert_eq!(
            tree["letterSpacing"]["value"].as_f64(),
            Some(5.0)
        );
    }

    #[test]
    fn test_preserve_non_default_line_height() {
        let mut tree = json!({
            "name": "Text",
            "lineHeight": {"units": "PERCENT", "value": 120.0},
            "fontSize": 16.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        // Non-default lineHeight should be preserved
        assert!(tree.get("lineHeight").is_some());
        assert_eq!(
            tree["lineHeight"]["value"].as_f64(),
            Some(120.0)
        );
    }

    #[test]
    fn test_preserve_pixels_units() {
        let mut tree = json!({
            "name": "Text",
            "letterSpacing": {"units": "PIXELS", "value": 0.0},
            "lineHeight": {"units": "PIXELS", "value": 100.0},
            "fontSize": 16.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        // Non-PERCENT units should be preserved even with default values
        assert!(tree.get("letterSpacing").is_some());
        assert!(tree.get("lineHeight").is_some());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Text1",
                    "letterSpacing": {"units": "PERCENT", "value": 0.0}
                },
                {
                    "name": "Text2",
                    "lineHeight": {"units": "PERCENT", "value": 100.0}
                }
            ]
        });

        remove_default_text_properties(&mut tree).unwrap();

        // Both nested defaults should be removed
        assert!(tree["children"][0].get("letterSpacing").is_none());
        assert!(tree["children"][1].get("lineHeight").is_none());
    }

    #[test]
    fn test_no_text_properties() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_text_properties(&mut tree).unwrap();

        // Tree without text properties should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "TEXT",
                        "letterSpacing": {"units": "PERCENT", "value": 0.0},
                        "lineHeight": {"units": "PERCENT", "value": 100.0}
                    }
                ]
            }
        });

        remove_default_text_properties(&mut tree).unwrap();

        let text_node = &tree["document"]["children"][0];
        assert!(text_node.get("letterSpacing").is_none());
        assert!(text_node.get("lineHeight").is_none());
        assert_eq!(text_node["type"].as_str(), Some("TEXT"));
    }

    #[test]
    fn test_is_default_letter_spacing() {
        assert!(is_default_letter_spacing(&json!({
            "units": "PERCENT",
            "value": 0.0
        })));

        assert!(!is_default_letter_spacing(&json!({
            "units": "PERCENT",
            "value": 5.0
        })));

        assert!(!is_default_letter_spacing(&json!({
            "units": "PIXELS",
            "value": 0.0
        })));

        assert!(!is_default_letter_spacing(&json!(0.0)));
    }

    #[test]
    fn test_is_default_line_height() {
        assert!(is_default_line_height(&json!({
            "units": "PERCENT",
            "value": 100.0
        })));

        assert!(!is_default_line_height(&json!({
            "units": "PERCENT",
            "value": 120.0
        })));

        assert!(!is_default_line_height(&json!({
            "units": "PIXELS",
            "value": 100.0
        })));

        assert!(!is_default_line_height(&json!(100.0)));
    }
}
