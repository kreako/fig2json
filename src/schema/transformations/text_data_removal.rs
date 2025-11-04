use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove textData field from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes the "textData" field which
/// contains Figma-specific text line metadata including:
/// - "lines" - Array of line metadata (indentationLevel, lineType, etc.)
/// - "characters" - Raw text characters
/// - Other Figma internal text representations
///
/// This field is only useful within Figma's internal rendering engine and is
/// not needed for HTML/CSS text rendering. The actual text content is already
/// available in other fields that are preserved.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all textData fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_text_data_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Text",
///     "textData": {
///         "characters": "Hello",
///         "lines": [{"lineType": "PLAIN"}]
///     },
///     "fontSize": 16.0
/// });
/// remove_text_data_fields(&mut tree).unwrap();
/// // tree now has only "name" and "fontSize" fields
/// ```
pub fn remove_text_data_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove textData fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove textData field if it exists
            map.remove("textData");

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
    fn test_remove_text_data() {
        let mut tree = json!({
            "name": "Text",
            "textData": {
                "characters": "Hello World",
                "lines": [
                    {
                        "lineType": "PLAIN",
                        "indentationLevel": 0
                    }
                ]
            },
            "fontSize": 16.0
        });

        remove_text_data_fields(&mut tree).unwrap();

        assert!(tree.get("textData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_preserve_other_fields() {
        let mut tree = json!({
            "name": "TextNode",
            "visible": true,
            "textData": {
                "characters": "Test",
                "lines": []
            },
            "fontSize": 14,
            "fontFamily": "Arial"
        });

        remove_text_data_fields(&mut tree).unwrap();

        // Check that non-textData fields are preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("TextNode"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("fontSize").unwrap().as_i64(), Some(14));
        assert_eq!(tree.get("fontFamily").unwrap().as_str(), Some("Arial"));
        assert!(tree.get("textData").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "textData": {
                        "characters": "Text1",
                        "lines": [{"lineType": "PLAIN"}]
                    }
                },
                {
                    "name": "Child2",
                    "children": [
                        {
                            "name": "DeepChild",
                            "textData": {
                                "characters": "Text2",
                                "lines": []
                            }
                        }
                    ]
                }
            ]
        });

        remove_text_data_fields(&mut tree).unwrap();

        // Check all nested textData removed
        assert!(tree["children"][0].get("textData").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));
        assert!(tree["children"][1]["children"][0].get("textData").is_none());
        assert_eq!(
            tree["children"][1]["children"][0]["name"].as_str(),
            Some("DeepChild")
        );
    }

    #[test]
    fn test_no_text_data() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "fills": []
        });

        remove_text_data_fields(&mut tree).unwrap();

        // Tree without textData should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert!(tree.get("textData").is_none());
    }

    #[test]
    fn test_multiple_text_data() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Text1",
                    "textData": {"characters": "One"}
                },
                {
                    "name": "Text2",
                    "textData": {"characters": "Two"}
                },
                {
                    "name": "Text3",
                    "textData": {"characters": "Three"}
                }
            ]
        });

        remove_text_data_fields(&mut tree).unwrap();

        // All textData fields should be removed
        assert!(tree["children"][0].get("textData").is_none());
        assert!(tree["children"][1].get("textData").is_none());
        assert!(tree["children"][2].get("textData").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Text1"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Text2"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("Text3"));
    }

    #[test]
    fn test_empty_text_data() {
        let mut tree = json!({
            "name": "Text",
            "textData": {},
            "fontSize": 12
        });

        remove_text_data_fields(&mut tree).unwrap();

        assert!(tree.get("textData").is_none());
        assert_eq!(tree.get("fontSize").unwrap().as_i64(), Some(12));
    }

    #[test]
    fn test_text_data_in_array() {
        let mut tree = json!({
            "elements": [
                {
                    "type": "text",
                    "textData": {"characters": "A"}
                },
                {
                    "type": "text",
                    "textData": {"characters": "B"}
                }
            ]
        });

        remove_text_data_fields(&mut tree).unwrap();

        // All textData in arrays should be removed
        assert!(tree["elements"][0].get("textData").is_none());
        assert!(tree["elements"][1].get("textData").is_none());
        assert_eq!(tree["elements"][0]["type"].as_str(), Some("text"));
        assert_eq!(tree["elements"][1]["type"].as_str(), Some("text"));
    }

    #[test]
    fn test_deeply_nested_text_data() {
        let mut tree = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "textData": {
                                "characters": "Deep",
                                "lines": [{"lineType": "PLAIN"}]
                            },
                            "name": "DeepNode"
                        }
                    }
                }
            }
        });

        remove_text_data_fields(&mut tree).unwrap();

        // Deeply nested textData should be removed
        let deep_node = &tree["level1"]["level2"]["level3"]["level4"];
        assert!(deep_node.get("textData").is_none());
        assert_eq!(deep_node["name"].as_str(), Some("DeepNode"));
    }
}
