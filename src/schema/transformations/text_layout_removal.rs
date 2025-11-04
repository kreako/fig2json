use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove detailed text layout data from derivedTextData objects
///
/// Recursively traverses the JSON tree and removes detailed layout fields from
/// "derivedTextData" objects:
/// - "baselines" - Precise line baseline positioning
/// - "logicalIndexToCharacterOffsetMap" - Character position map
/// - "fontMetaData" - Font digest and metadata arrays
/// - "derivedLines" - Line directionality information
/// - "truncatedHeight" - Truncation height value
/// - "truncationStartIndex" - Truncation start index
///
/// These fields contain precise text layout data that is not needed for
/// basic HTML/CSS text rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all text layout fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_text_layout_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "derivedTextData": {
///         "baselines": [{"lineY": 10.0, "width": 100.0}],
///         "logicalIndexToCharacterOffsetMap": [0.0, 10.0, 20.0],
///         "fontMetaData": [{"fontDigest": [1, 2, 3]}],
///         "layoutSize": {"x": 100.0, "y": 50.0}
///     }
/// });
/// remove_text_layout_fields(&mut tree).unwrap();
/// // derivedTextData now has only "layoutSize"
/// ```
pub fn remove_text_layout_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove text layout fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check if this object has a "derivedTextData" field
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "derivedTextData" {
                    // This field might contain layout data to remove
                    if let Some(derived_text_data) = map.get_mut(&key) {
                        if let Some(obj) = derived_text_data.as_object_mut() {
                            // Remove all the detailed layout fields
                            obj.remove("baselines");
                            obj.remove("logicalIndexToCharacterOffsetMap");
                            obj.remove("fontMetaData");
                            obj.remove("derivedLines");
                            obj.remove("truncatedHeight");
                            obj.remove("truncationStartIndex");
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
    fn test_remove_baselines() {
        let mut tree = json!({
            "derivedTextData": {
                "baselines": [
                    {
                        "endCharacter": 5,
                        "firstCharacter": 0,
                        "lineAscent": 124.0,
                        "lineHeight": 155.0,
                        "lineY": 1.3871626833861228e-6,
                        "position": {"x": 0.0, "y": 124.04545593261719},
                        "width": 306.375
                    }
                ],
                "layoutSize": {"x": 307.0, "y": 155.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("baselines").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_remove_character_offset_map() {
        let mut tree = json!({
            "derivedTextData": {
                "logicalIndexToCharacterOffsetMap": [0.0, 94.75, 169.25, 199.625, 230.0],
                "layoutSize": {"x": 307.0, "y": 155.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data
            .get("logicalIndexToCharacterOffsetMap")
            .is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_remove_font_metadata() {
        let mut tree = json!({
            "derivedTextData": {
                "fontMetaData": [
                    {
                        "fontDigest": [212, 131, 226, 199],
                        "fontLineHeight": 1.2102272510528564,
                        "fontStyle": {"__enum__": "FontStyle", "value": "NORMAL"},
                        "fontWeight": 400,
                        "key": {
                            "family": "Inter",
                            "postscript": "",
                            "style": "Regular"
                        }
                    }
                ],
                "layoutSize": {"x": 100.0, "y": 50.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("fontMetaData").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_remove_derived_lines() {
        let mut tree = json!({
            "derivedTextData": {
                "derivedLines": [
                    {
                        "directionality": {"__enum__": "Directionality", "value": "LTR"}
                    }
                ],
                "layoutSize": {"x": 100.0, "y": 50.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("derivedLines").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_remove_truncation_fields() {
        let mut tree = json!({
            "derivedTextData": {
                "truncatedHeight": 100.0,
                "truncationStartIndex": 42,
                "layoutSize": {"x": 100.0, "y": 50.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("truncatedHeight").is_none());
        assert!(derived_text_data.get("truncationStartIndex").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_remove_all_layout_fields() {
        let mut tree = json!({
            "derivedTextData": {
                "baselines": [{"lineY": 10.0}],
                "logicalIndexToCharacterOffsetMap": [0.0, 10.0],
                "fontMetaData": [{"fontDigest": [1, 2, 3]}],
                "derivedLines": [{"directionality": {"__enum__": "Directionality", "value": "LTR"}}],
                "truncatedHeight": -1.0,
                "truncationStartIndex": -1,
                "layoutSize": {"x": 100.0, "y": 50.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("baselines").is_none());
        assert!(derived_text_data
            .get("logicalIndexToCharacterOffsetMap")
            .is_none());
        assert!(derived_text_data.get("fontMetaData").is_none());
        assert!(derived_text_data.get("derivedLines").is_none());
        assert!(derived_text_data.get("truncatedHeight").is_none());
        assert!(derived_text_data.get("truncationStartIndex").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_preserve_other_derived_text_data_fields() {
        let mut tree = json!({
            "name": "TextNode",
            "derivedTextData": {
                "baselines": [{"lineY": 10.0}],
                "layoutSize": {"x": 100.0, "y": 50.0},
                "customField": "preserved"
            },
            "visible": true
        });

        remove_text_layout_fields(&mut tree).unwrap();

        // Check that non-layout fields are preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("TextNode"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));

        // Check that derivedTextData preserves non-layout fields
        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("baselines").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
        assert_eq!(
            derived_text_data.get("customField").unwrap().as_str(),
            Some("preserved")
        );
    }

    #[test]
    fn test_nested_derived_text_data() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "derivedTextData": {
                        "baselines": [{"lineY": 10.0}],
                        "layoutSize": {"x": 100.0, "y": 50.0}
                    }
                },
                {
                    "name": "Child2",
                    "children": [
                        {
                            "name": "DeepChild",
                            "derivedTextData": {
                                "fontMetaData": [{"fontDigest": [1, 2, 3]}],
                                "layoutSize": {"x": 200.0, "y": 100.0}
                            }
                        }
                    ]
                }
            ]
        });

        remove_text_layout_fields(&mut tree).unwrap();

        // Check first nested derivedTextData
        let child1_data = &tree["children"][0]["derivedTextData"];
        assert!(child1_data.get("baselines").is_none());
        assert!(child1_data.get("layoutSize").is_some());

        // Check deeply nested derivedTextData
        let deep_child_data = &tree["children"][1]["children"][0]["derivedTextData"];
        assert!(deep_child_data.get("fontMetaData").is_none());
        assert!(deep_child_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_no_derived_text_data() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_text_layout_fields(&mut tree).unwrap();

        // Tree without derivedTextData should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert!(tree.get("derivedTextData").is_none());
    }

    #[test]
    fn test_empty_derived_text_data() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {}
        });

        remove_text_layout_fields(&mut tree).unwrap();

        // Empty derivedTextData should remain empty
        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert_eq!(derived_text_data.as_object().unwrap().len(), 0);
    }
}
