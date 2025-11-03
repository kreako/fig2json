use crate::error::Result;
use serde_json::Value as JsonValue;

/// Transform RGBA color objects to CSS hex color strings
///
/// Recursively traverses the JSON tree and transforms any object with r, g, b
/// (and optionally a) fields by:
/// - Converting float values (0.0-1.0) to hex bytes (00-ff)
/// - Replacing the entire object with a hex string: "#rrggbb" or "#rrggbbaa"
/// - Using #rrggbb format when alpha is 1.0 or missing
/// - Using #rrggbbaa format when alpha is not 1.0
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully transformed all color objects
///
/// # Examples
/// ```no_run
/// use fig2json::schema::transform_colors_to_css;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "color": {
///         "r": 0.8725961446762085,
///         "g": 0.06292760372161865,
///         "b": 0.06292760372161865,
///         "a": 1.0
///     }
/// });
/// transform_colors_to_css(&mut tree).unwrap();
/// // tree now has "color": "#df1010"
/// ```
pub fn transform_colors_to_css(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively transform color objects in a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Collect keys to avoid borrow checker issues
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if let Some(val) = map.get(&key) {
                    // Check if this value is a color object
                    if let Some(obj) = val.as_object() {
                        if is_color_object(obj) {
                            // Convert color object to CSS hex string
                            if let Some(css_color) = convert_color_to_css(obj) {
                                map.insert(key.clone(), JsonValue::String(css_color));
                                continue; // Skip recursion since we replaced the object
                            }
                        }
                    }
                }

                // Recurse into the value if it wasn't replaced
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

/// Check if an object is a color object (has r, g, b fields)
fn is_color_object(obj: &serde_json::Map<String, JsonValue>) -> bool {
    obj.contains_key("r") && obj.contains_key("g") && obj.contains_key("b")
}

/// Convert a color object to CSS hex string
///
/// Converts RGBA values (0.0-1.0 range) to hex format.
/// Returns #rrggbb if alpha is 1.0 or missing, #rrggbbaa otherwise.
///
/// # Arguments
/// * `obj` - Color object with r, g, b, and optionally a fields
///
/// # Returns
/// * `Some(String)` - The CSS hex color string
/// * `None` - If any required field is missing or not a valid f64
fn convert_color_to_css(obj: &serde_json::Map<String, JsonValue>) -> Option<String> {
    // Extract r, g, b values (required)
    let r = obj.get("r")?.as_f64()?;
    let g = obj.get("g")?.as_f64()?;
    let b = obj.get("b")?.as_f64()?;

    // Extract alpha value (optional, defaults to 1.0)
    let a = obj.get("a").and_then(|v| v.as_f64()).unwrap_or(1.0);

    // Convert 0.0-1.0 range to 0-255 range
    let r_byte = float_to_byte(r);
    let g_byte = float_to_byte(g);
    let b_byte = float_to_byte(b);
    let a_byte = float_to_byte(a);

    // Format as hex string
    // Use #rrggbb format when alpha is 1.0 (fully opaque)
    // Use #rrggbbaa format when alpha is not 1.0
    if (a - 1.0).abs() < 0.001 {
        // Alpha is approximately 1.0, use 6-character format
        Some(format!("#{:02x}{:02x}{:02x}", r_byte, g_byte, b_byte))
    } else {
        // Alpha is not 1.0, include it in 8-character format
        Some(format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            r_byte, g_byte, b_byte, a_byte
        ))
    }
}

/// Convert a float in range 0.0-1.0 to a byte in range 0-255
///
/// Clamps the input to [0.0, 1.0] range and rounds to nearest integer.
fn float_to_byte(value: f64) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_float_to_byte() {
        assert_eq!(float_to_byte(0.0), 0);
        assert_eq!(float_to_byte(1.0), 255);
        assert_eq!(float_to_byte(0.5), 128);
        assert_eq!(float_to_byte(0.8725961446762085), 223); // From example
        assert_eq!(float_to_byte(0.06292760372161865), 16);
    }

    #[test]
    fn test_float_to_byte_clamping() {
        assert_eq!(float_to_byte(-0.5), 0); // Negative clamped to 0
        assert_eq!(float_to_byte(1.5), 255); // Over 1.0 clamped to 255
    }

    #[test]
    fn test_is_color_object() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.5,
            "g": 0.5,
            "b": 0.5,
            "a": 1.0
        }))
        .unwrap();
        assert!(is_color_object(&color));

        let color_no_alpha = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.5,
            "g": 0.5,
            "b": 0.5
        }))
        .unwrap();
        assert!(is_color_object(&color_no_alpha));

        let not_color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "x": 10,
            "y": 20
        }))
        .unwrap();
        assert!(!is_color_object(&not_color));

        let incomplete_color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.5,
            "g": 0.5
        }))
        .unwrap();
        assert!(!is_color_object(&incomplete_color));
    }

    #[test]
    fn test_convert_color_to_css_opaque() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.8725961446762085,
            "g": 0.06292760372161865,
            "b": 0.06292760372161865,
            "a": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#df1010");
    }

    #[test]
    fn test_convert_color_to_css_transparent() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 1.0,
            "g": 0.0,
            "b": 0.0,
            "a": 0.5
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#ff000080");
    }

    #[test]
    fn test_convert_color_to_css_no_alpha() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.0,
            "g": 0.5,
            "b": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#0080ff");
    }

    #[test]
    fn test_convert_color_to_css_black() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.0,
            "g": 0.0,
            "b": 0.0,
            "a": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#000000");
    }

    #[test]
    fn test_convert_color_to_css_white() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 1.0,
            "g": 1.0,
            "b": 1.0,
            "a": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#ffffff");
    }

    #[test]
    fn test_transform_simple_color() {
        let mut tree = json!({
            "name": "Rectangle",
            "color": {
                "r": 0.8725961446762085,
                "g": 0.06292760372161865,
                "b": 0.06292760372161865,
                "a": 1.0
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree.get("color").unwrap().as_str(), Some("#df1010"));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_transform_multiple_colors() {
        let mut tree = json!({
            "backgroundColor": {
                "r": 1.0,
                "g": 0.0,
                "b": 0.0,
                "a": 1.0
            },
            "foregroundColor": {
                "r": 0.0,
                "g": 1.0,
                "b": 0.0,
                "a": 0.5
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(
            tree.get("backgroundColor").unwrap().as_str(),
            Some("#ff0000")
        );
        assert_eq!(
            tree.get("foregroundColor").unwrap().as_str(),
            Some("#00ff0080")
        );
    }

    #[test]
    fn test_transform_nested_colors() {
        let mut tree = json!({
            "name": "Root",
            "style": {
                "fill": {
                    "r": 1.0,
                    "g": 0.0,
                    "b": 0.0,
                    "a": 1.0
                },
                "stroke": {
                    "r": 0.0,
                    "g": 0.0,
                    "b": 1.0,
                    "a": 0.8
                }
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree["style"]["fill"].as_str(), Some("#ff0000"));
        assert_eq!(tree["style"]["stroke"].as_str(), Some("#0000ffcc"));
    }

    #[test]
    fn test_transform_colors_in_array() {
        let mut tree = json!({
            "fills": [
                {
                    "type": "solid",
                    "color": {
                        "r": 1.0,
                        "g": 0.0,
                        "b": 0.0,
                        "a": 1.0
                    }
                },
                {
                    "type": "solid",
                    "color": {
                        "r": 0.0,
                        "g": 1.0,
                        "b": 0.0,
                        "a": 0.5
                    }
                }
            ]
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree["fills"][0]["color"].as_str(), Some("#ff0000"));
        assert_eq!(tree["fills"][1]["color"].as_str(), Some("#00ff0080"));
    }

    #[test]
    fn test_transform_preserves_non_color_objects() {
        let mut tree = json!({
            "name": "Rectangle",
            "position": {
                "x": 10,
                "y": 20
            },
            "color": {
                "r": 1.0,
                "g": 0.0,
                "b": 0.0,
                "a": 1.0
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        // Color should be transformed
        assert_eq!(tree.get("color").unwrap().as_str(), Some("#ff0000"));

        // Position should be unchanged
        assert_eq!(tree["position"]["x"].as_i64(), Some(10));
        assert_eq!(tree["position"]["y"].as_i64(), Some(20));

        // Name should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_transform_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "name": "Frame",
                        "fills": [
                            {
                                "type": "solid",
                                "color": {
                                    "r": 0.5,
                                    "g": 0.5,
                                    "b": 0.5,
                                    "a": 1.0
                                }
                            }
                        ]
                    }
                ]
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(
            tree["document"]["children"][0]["fills"][0]["color"]
                .as_str(),
            Some("#808080")
        );
    }
}
