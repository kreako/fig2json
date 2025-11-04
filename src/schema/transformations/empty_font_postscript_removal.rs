use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove empty postscript field from fontName objects
///
/// Recursively traverses the JSON tree and removes the "postscript" field from
/// "fontName" objects when it is an empty string. The postscript field is used
/// to specify the PostScript font name, but when empty it provides no information
/// and can be safely removed to reduce JSON size.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all empty postscript fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_empty_font_postscript;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "fontName": {
///         "family": "Inter",
///         "style": "Regular",
///         "postscript": ""
///     }
/// });
/// remove_empty_font_postscript(&mut tree).unwrap();
/// // fontName now has only "family" and "style" fields
/// ```
pub fn remove_empty_font_postscript(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove empty postscript fields from fontName objects
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check if this object has a "fontName" field
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "fontName" {
                    // This might be a fontName object with postscript field
                    if let Some(font_name) = map.get_mut(&key) {
                        if let Some(font_obj) = font_name.as_object_mut() {
                            // Check if postscript exists and is empty
                            if let Some(postscript) = font_obj.get("postscript") {
                                if let Some(s) = postscript.as_str() {
                                    if s.is_empty() {
                                        font_obj.remove("postscript");
                                    }
                                }
                            }
                        }
                    }
                }

                // Recurse into the value regardless
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
    fn test_remove_empty_postscript() {
        let mut tree = json!({
            "name": "Text",
            "fontName": {
                "family": "Inter",
                "style": "Regular",
                "postscript": ""
            }
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        let font_name = tree.get("fontName").unwrap();
        assert!(font_name.get("postscript").is_none());
        assert_eq!(font_name.get("family").unwrap().as_str(), Some("Inter"));
        assert_eq!(font_name.get("style").unwrap().as_str(), Some("Regular"));
    }

    #[test]
    fn test_preserve_non_empty_postscript() {
        let mut tree = json!({
            "name": "Text",
            "fontName": {
                "family": "Helvetica",
                "style": "Bold",
                "postscript": "Helvetica-Bold"
            }
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        let font_name = tree.get("fontName").unwrap();
        // Non-empty postscript should be preserved
        assert_eq!(
            font_name.get("postscript").unwrap().as_str(),
            Some("Helvetica-Bold")
        );
        assert_eq!(font_name.get("family").unwrap().as_str(), Some("Helvetica"));
        assert_eq!(font_name.get("style").unwrap().as_str(), Some("Bold"));
    }

    #[test]
    fn test_no_postscript_field() {
        let mut tree = json!({
            "name": "Text",
            "fontName": {
                "family": "Arial",
                "style": "Regular"
            }
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        let font_name = tree.get("fontName").unwrap();
        // fontName without postscript should be unchanged
        assert!(font_name.get("postscript").is_none());
        assert_eq!(font_name.get("family").unwrap().as_str(), Some("Arial"));
        assert_eq!(font_name.get("style").unwrap().as_str(), Some("Regular"));
    }

    #[test]
    fn test_no_font_name() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        // Tree without fontName should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("fontName").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Text1",
                    "fontName": {
                        "family": "Inter",
                        "postscript": ""
                    }
                },
                {
                    "name": "Text2",
                    "fontName": {
                        "family": "Roboto",
                        "postscript": ""
                    }
                }
            ]
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        // Both empty postscripts should be removed
        assert!(tree["children"][0]["fontName"].get("postscript").is_none());
        assert_eq!(
            tree["children"][0]["fontName"]["family"].as_str(),
            Some("Inter")
        );
        assert!(tree["children"][1]["fontName"].get("postscript").is_none());
        assert_eq!(
            tree["children"][1]["fontName"]["family"].as_str(),
            Some("Roboto")
        );
    }

    #[test]
    fn test_mixed_empty_and_non_empty() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Text1",
                    "fontName": {
                        "family": "Inter",
                        "postscript": ""
                    }
                },
                {
                    "name": "Text2",
                    "fontName": {
                        "family": "Helvetica",
                        "postscript": "Helvetica-Bold"
                    }
                }
            ]
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        // Empty postscript removed, non-empty preserved
        assert!(tree["children"][0]["fontName"].get("postscript").is_none());
        assert_eq!(
            tree["children"][1]["fontName"]["postscript"].as_str(),
            Some("Helvetica-Bold")
        );
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "TEXT",
                        "fontName": {
                            "family": "Times",
                            "style": "Italic",
                            "postscript": ""
                        }
                    }
                ]
            }
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        let font_name = &tree["document"]["children"][0]["fontName"];
        assert!(font_name.get("postscript").is_none());
        assert_eq!(font_name["family"].as_str(), Some("Times"));
        assert_eq!(font_name["style"].as_str(), Some("Italic"));
    }

    #[test]
    fn test_postscript_non_string() {
        let mut tree = json!({
            "fontName": {
                "family": "Test",
                "postscript": 123
            }
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        // Non-string postscript should be preserved
        let font_name = tree.get("fontName").unwrap();
        assert!(font_name.get("postscript").is_some());
        assert_eq!(font_name.get("postscript").unwrap().as_i64(), Some(123));
    }
}
