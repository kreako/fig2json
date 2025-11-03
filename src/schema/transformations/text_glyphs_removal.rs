use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove glyph vector data from text objects
///
/// Recursively traverses the JSON tree and removes "glyphs" arrays from
/// "derivedTextData" objects. Text glyph vector paths (M, L, Q, Z commands)
/// are not needed in the JSON output, so removing them significantly reduces
/// file size.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all text glyphs
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_text_glyphs;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "derivedTextData": {
///         "glyphs": [
///             {"advance": 0.74, "commands": ["Z", "M"]}
///         ],
///         "layoutInfo": "preserved"
///     }
/// });
/// remove_text_glyphs(&mut tree).unwrap();
/// // tree now has "derivedTextData": {"layoutInfo": "preserved"}
/// ```
pub fn remove_text_glyphs(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove text glyphs from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check if this object has a "derivedTextData" field
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "derivedTextData" {
                    // This field might contain glyphs to remove
                    if let Some(derived_text_data) = map.get_mut(&key) {
                        if let Some(obj) = derived_text_data.as_object_mut() {
                            // Remove the "glyphs" field if it exists
                            obj.remove("glyphs");
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
    fn test_remove_glyphs_from_derived_text_data() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {
                "glyphs": [
                    {"advance": 0.74, "commands": ["Z", "M", "L", "Q"]},
                    {"advance": 0.82, "commands": ["M", "L"]}
                ],
                "layoutInfo": {"width": 100, "height": 20}
            }
        });

        remove_text_glyphs(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("glyphs").is_none());
        assert!(derived_text_data.get("layoutInfo").is_some());
    }

    #[test]
    fn test_preserve_other_fields() {
        let mut tree = json!({
            "name": "TextNode",
            "visible": true,
            "derivedTextData": {
                "glyphs": [{"advance": 0.5, "commands": ["M", "Z"]}],
                "layoutInfo": {"baseline": 12},
                "fontFamily": "Arial",
                "fontSize": 14
            },
            "x": 10,
            "y": 20
        });

        remove_text_glyphs(&mut tree).unwrap();

        // Check that non-derivedTextData fields are preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("TextNode"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));

        // Check that derivedTextData preserves all fields except glyphs
        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("glyphs").is_none());
        assert_eq!(
            derived_text_data.get("layoutInfo").unwrap().get("baseline").unwrap().as_i64(),
            Some(12)
        );
        assert_eq!(
            derived_text_data.get("fontFamily").unwrap().as_str(),
            Some("Arial")
        );
        assert_eq!(
            derived_text_data.get("fontSize").unwrap().as_i64(),
            Some(14)
        );
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "derivedTextData": {
                        "glyphs": [{"advance": 0.6}],
                        "data": "keep"
                    }
                },
                {
                    "name": "Child2",
                    "children": [
                        {
                            "name": "DeepChild",
                            "derivedTextData": {
                                "glyphs": [{"advance": 0.8}],
                                "info": "preserve"
                            }
                        }
                    ]
                }
            ]
        });

        remove_text_glyphs(&mut tree).unwrap();

        // Check first nested derivedTextData
        let child1_data = &tree["children"][0]["derivedTextData"];
        assert!(child1_data.get("glyphs").is_none());
        assert_eq!(child1_data.get("data").unwrap().as_str(), Some("keep"));

        // Check deeply nested derivedTextData
        let deep_child_data = &tree["children"][1]["children"][0]["derivedTextData"];
        assert!(deep_child_data.get("glyphs").is_none());
        assert_eq!(deep_child_data.get("info").unwrap().as_str(), Some("preserve"));
    }

    #[test]
    fn test_no_glyphs_field() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {
                "layoutInfo": {"width": 100},
                "fontFamily": "Helvetica"
            }
        });

        remove_text_glyphs(&mut tree).unwrap();

        // derivedTextData without glyphs should be unchanged
        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("glyphs").is_none());
        assert!(derived_text_data.get("layoutInfo").is_some());
        assert_eq!(
            derived_text_data.get("fontFamily").unwrap().as_str(),
            Some("Helvetica")
        );
    }

    #[test]
    fn test_no_derived_text_data() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "fills": []
        });

        remove_text_glyphs(&mut tree).unwrap();

        // Tree without derivedTextData should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert!(tree.get("derivedTextData").is_none());
    }

    #[test]
    fn test_multiple_derived_text_data() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Text1",
                    "derivedTextData": {
                        "glyphs": [{"advance": 0.5}],
                        "prop1": "value1"
                    }
                },
                {
                    "name": "Text2",
                    "derivedTextData": {
                        "glyphs": [{"advance": 0.7}],
                        "prop2": "value2"
                    }
                }
            ]
        });

        remove_text_glyphs(&mut tree).unwrap();

        // All derivedTextData objects should have glyphs removed
        let text1_data = &tree["children"][0]["derivedTextData"];
        assert!(text1_data.get("glyphs").is_none());
        assert_eq!(text1_data.get("prop1").unwrap().as_str(), Some("value1"));

        let text2_data = &tree["children"][1]["derivedTextData"];
        assert!(text2_data.get("glyphs").is_none());
        assert_eq!(text2_data.get("prop2").unwrap().as_str(), Some("value2"));
    }

    #[test]
    fn test_empty_glyphs_array() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {
                "glyphs": [],
                "info": "test"
            }
        });

        remove_text_glyphs(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("glyphs").is_none());
        assert_eq!(derived_text_data.get("info").unwrap().as_str(), Some("test"));
    }

    #[test]
    fn test_glyphs_in_other_contexts_preserved() {
        let mut tree = json!({
            "name": "Root",
            "metadata": {
                "glyphs": [{"some": "data"}]
            },
            "derivedTextData": {
                "glyphs": [{"advance": 0.5}],
                "info": "test"
            }
        });

        remove_text_glyphs(&mut tree).unwrap();

        // Only glyphs inside derivedTextData should be removed
        assert!(tree.get("metadata").unwrap().get("glyphs").is_some());

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("glyphs").is_none());
    }
}
