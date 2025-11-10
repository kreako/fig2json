use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove default text line properties from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes line properties within
/// `textData.lines` and `textValue.lines` arrays that have default values:
/// - "indentationLevel" with value 0 (no indentation)
/// - "isFirstLineOfList" with value false (not a list item)
/// - "lineType" with value "PLAIN" (plain text)
/// - "listStartOffset" with value 0 (no list offset)
/// - "sourceDirectionality" with value "AUTO" (automatic text direction)
/// - "styleId" with value 0 (no style applied)
///
/// If all line objects in a `lines` array become empty after removing defaults,
/// the entire `lines` array is removed.
///
/// These are the default values in Figma for plain text rendering, so omitting
/// them reduces output size without losing information for HTML/CSS conversion.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all default text line fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_default_text_line_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "textData": {
///         "characters": "Hello",
///         "lines": [
///             {
///                 "indentationLevel": 0,
///                 "isFirstLineOfList": false,
///                 "lineType": "PLAIN",
///                 "listStartOffset": 0,
///                 "sourceDirectionality": "AUTO",
///                 "styleId": 0
///             }
///         ]
///     }
/// });
/// remove_default_text_line_properties(&mut tree).unwrap();
/// // The entire "lines" array is removed because all values were defaults
/// ```
pub fn remove_default_text_line_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove default text line properties from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Check if this object has a "lines" array
            if let Some(lines_value) = map.get_mut("lines") {
                if let Some(lines_array) = lines_value.as_array_mut() {
                    // Process each line object in the array
                    for line in lines_array.iter_mut() {
                        if let Some(line_obj) = line.as_object_mut() {
                            remove_default_line_fields(line_obj);
                        }
                    }

                    // Check if all lines are now empty objects
                    // Only remove the lines array if it has elements and all of them are empty
                    let all_empty = !lines_array.is_empty()
                        && lines_array.iter().all(|line| {
                            line.as_object()
                                .map(|obj| obj.is_empty())
                                .unwrap_or(false)
                        });

                    // If all lines are empty, remove the entire "lines" array
                    if all_empty {
                        map.remove("lines");
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

/// Remove default-valued fields from a single line object
fn remove_default_line_fields(line_obj: &mut serde_json::Map<String, JsonValue>) {
    // Remove indentationLevel if 0
    if let Some(val) = line_obj.get("indentationLevel") {
        if val.as_i64() == Some(0) {
            line_obj.remove("indentationLevel");
        }
    }

    // Remove isFirstLineOfList if false
    if let Some(val) = line_obj.get("isFirstLineOfList") {
        if val.as_bool() == Some(false) {
            line_obj.remove("isFirstLineOfList");
        }
    }

    // Remove lineType if "PLAIN"
    if let Some(val) = line_obj.get("lineType") {
        if val.as_str() == Some("PLAIN") {
            line_obj.remove("lineType");
        }
    }

    // Remove listStartOffset if 0
    if let Some(val) = line_obj.get("listStartOffset") {
        if val.as_i64() == Some(0) {
            line_obj.remove("listStartOffset");
        }
    }

    // Remove sourceDirectionality if "AUTO"
    if let Some(val) = line_obj.get("sourceDirectionality") {
        if val.as_str() == Some("AUTO") {
            line_obj.remove("sourceDirectionality");
        }
    }

    // Remove styleId if 0
    if let Some(val) = line_obj.get("styleId") {
        if val.as_i64() == Some(0) {
            line_obj.remove("styleId");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_all_defaults_removes_lines_array() {
        let mut tree = json!({
            "textData": {
                "characters": "Hello",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Entire lines array should be removed
        assert!(tree["textData"].get("lines").is_none());
        assert_eq!(
            tree["textData"]["characters"].as_str(),
            Some("Hello")
        );
    }

    #[test]
    fn test_preserve_non_default_indentation_level() {
        let mut tree = json!({
            "textData": {
                "characters": "Indented text",
                "lines": [
                    {
                        "indentationLevel": 2,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Lines array should still exist because indentationLevel is non-default
        assert!(tree["textData"].get("lines").is_some());
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["indentationLevel"].as_i64(), Some(2));
        // All other defaults should be removed
        assert!(lines[0].get("isFirstLineOfList").is_none());
        assert!(lines[0].get("lineType").is_none());
        assert!(lines[0].get("listStartOffset").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
        assert!(lines[0].get("styleId").is_none());
    }

    #[test]
    fn test_preserve_list_item() {
        let mut tree = json!({
            "textData": {
                "characters": "• List item",
                "lines": [
                    {
                        "indentationLevel": 1,
                        "isFirstLineOfList": true,
                        "lineType": "UNORDERED_LIST",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Lines array should still exist
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["indentationLevel"].as_i64(), Some(1));
        assert_eq!(lines[0]["isFirstLineOfList"].as_bool(), Some(true));
        assert_eq!(lines[0]["lineType"].as_str(), Some("UNORDERED_LIST"));
        // Defaults should be removed
        assert!(lines[0].get("listStartOffset").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
        assert!(lines[0].get("styleId").is_none());
    }

    #[test]
    fn test_preserve_non_zero_style_id() {
        let mut tree = json!({
            "textData": {
                "characters": "Styled text",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 5
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Lines array should still exist because styleId is non-default
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["styleId"].as_i64(), Some(5));
        // All other defaults should be removed
        assert!(lines[0].get("indentationLevel").is_none());
        assert!(lines[0].get("isFirstLineOfList").is_none());
        assert!(lines[0].get("lineType").is_none());
        assert!(lines[0].get("listStartOffset").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
    }

    #[test]
    fn test_multiple_lines_mixed() {
        let mut tree = json!({
            "textData": {
                "characters": "Multi-line text",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    },
                    {
                        "indentationLevel": 1,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Lines array should still exist because second line has non-default indentation
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 2);
        // First line should be empty (all defaults)
        assert!(lines[0].as_object().unwrap().is_empty());
        // Second line should have only indentationLevel
        assert_eq!(lines[1]["indentationLevel"].as_i64(), Some(1));
        assert!(lines[1].get("isFirstLineOfList").is_none());
    }

    #[test]
    fn test_nested_text_data() {
        let mut tree = json!({
            "children": [
                {
                    "textData": {
                        "characters": "First",
                        "lines": [
                            {
                                "indentationLevel": 0,
                                "isFirstLineOfList": false,
                                "lineType": "PLAIN",
                                "listStartOffset": 0,
                                "sourceDirectionality": "AUTO",
                                "styleId": 0
                            }
                        ]
                    }
                },
                {
                    "textData": {
                        "characters": "Second",
                        "lines": [
                            {
                                "indentationLevel": 1,
                                "isFirstLineOfList": false,
                                "lineType": "PLAIN",
                                "listStartOffset": 0,
                                "sourceDirectionality": "AUTO",
                                "styleId": 0
                            }
                        ]
                    }
                }
            ]
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // First child should have lines array removed (all defaults)
        assert!(tree["children"][0]["textData"].get("lines").is_none());

        // Second child should still have lines array (indentation is non-default)
        assert!(tree["children"][1]["textData"].get("lines").is_some());
        let lines = tree["children"][1]["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines[0]["indentationLevel"].as_i64(), Some(1));
    }

    #[test]
    fn test_text_value_lines() {
        let mut tree = json!({
            "textValue": {
                "characters": "Hello",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Works for textValue.lines as well
        assert!(tree["textValue"].get("lines").is_none());
    }

    #[test]
    fn test_no_lines_field() {
        let mut tree = json!({
            "textData": {
                "characters": "Hello"
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Tree without lines should be unchanged
        assert_eq!(
            tree["textData"]["characters"].as_str(),
            Some("Hello")
        );
    }

    #[test]
    fn test_preserve_non_default_list_start_offset() {
        let mut tree = json!({
            "textData": {
                "characters": "Numbered list",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 5,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Lines array should still exist
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines[0]["listStartOffset"].as_i64(), Some(5));
        // Other defaults removed
        assert!(lines[0].get("indentationLevel").is_none());
        assert!(lines[0].get("isFirstLineOfList").is_none());
        assert!(lines[0].get("lineType").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
        assert!(lines[0].get("styleId").is_none());
    }

    #[test]
    fn test_deeply_nested_structure() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "TEXT",
                        "textData": {
                            "characters": "Deep text",
                            "lines": [
                                {
                                    "indentationLevel": 0,
                                    "isFirstLineOfList": false,
                                    "lineType": "PLAIN",
                                    "listStartOffset": 0,
                                    "sourceDirectionality": "AUTO",
                                    "styleId": 0
                                }
                            ]
                        }
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        let text_data = &tree["document"]["children"][0]["textData"];
        assert!(text_data.get("lines").is_none());
        assert_eq!(text_data["characters"].as_str(), Some("Deep text"));
    }

    #[test]
    fn test_empty_lines_array() {
        let mut tree = json!({
            "textData": {
                "characters": "Hello",
                "lines": []
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Empty lines array should be preserved (not our concern)
        assert!(tree["textData"].get("lines").is_some());
        assert_eq!(tree["textData"]["lines"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_multiple_all_default_lines() {
        let mut tree = json!({
            "textData": {
                "characters": "Multi-line",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    },
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    },
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // All lines are defaults, so entire lines array should be removed
        assert!(tree["textData"].get("lines").is_none());
    }
}
