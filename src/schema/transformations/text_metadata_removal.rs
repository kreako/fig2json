use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove text metadata fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes text configuration fields:
/// - "textBidiVersion" - Bidirectional text version
/// - "textExplicitLayoutVersion" - Explicit layout version
/// - "textUserLayoutVersion" - User layout version
/// - "textDecorationSkipInk" - Text decoration skip ink setting
/// - "fontVariantCommonLigatures" - Font ligature setting
/// - "fontVariantContextualLigatures" - Contextual ligature setting
/// - "fontVariations" - Font variation array
/// - "fontVersion" - Font version string
/// - "emojiImageSet" - Emoji image set enum
/// - "autoRename" - Auto rename flag
/// - "textTracking" - Text tracking value
/// - "textAlignVertical" - Vertical text alignment (Figma auto-layout)
/// - "textAutoResize" - Text auto-resize behavior (Figma auto-layout)
///
/// These fields contain text rendering configuration that is not needed for
/// basic HTML/CSS text rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all text metadata fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_text_metadata_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Text",
///     "textBidiVersion": 1,
///     "textUserLayoutVersion": 5,
///     "autoRename": true,
///     "fontSize": 16.0
/// });
/// remove_text_metadata_fields(&mut tree).unwrap();
/// // tree now has only "name" and "fontSize" fields
/// ```
pub fn remove_text_metadata_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove text metadata fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove all text metadata fields if they exist
            map.remove("textBidiVersion");
            map.remove("textExplicitLayoutVersion");
            map.remove("textUserLayoutVersion");
            map.remove("textDecorationSkipInk");
            map.remove("fontVariantCommonLigatures");
            map.remove("fontVariantContextualLigatures");
            map.remove("fontVariations");
            map.remove("fontVersion");
            map.remove("emojiImageSet");
            map.remove("autoRename");
            map.remove("textTracking");
            map.remove("textAlignVertical");
            map.remove("textAutoResize");

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
    fn test_remove_text_bidi_version() {
        let mut tree = json!({
            "name": "Text",
            "textBidiVersion": 1,
            "fontSize": 16.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("textBidiVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_remove_text_layout_versions() {
        let mut tree = json!({
            "name": "Text",
            "textExplicitLayoutVersion": 1,
            "textUserLayoutVersion": 5,
            "visible": true
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("textExplicitLayoutVersion").is_none());
        assert!(tree.get("textUserLayoutVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_font_variants() {
        let mut tree = json!({
            "name": "Text",
            "fontVariantCommonLigatures": true,
            "fontVariantContextualLigatures": true,
            "fontVariations": [],
            "fontVersion": "1.0",
            "fontSize": 14.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("fontVariantCommonLigatures").is_none());
        assert!(tree.get("fontVariantContextualLigatures").is_none());
        assert!(tree.get("fontVariations").is_none());
        assert!(tree.get("fontVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_remove_emoji_image_set() {
        let mut tree = json!({
            "name": "Text",
            "emojiImageSet": {
                "__enum__": "EmojiImageSet",
                "value": "APPLE"
            },
            "visible": true
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("emojiImageSet").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_auto_rename_and_tracking() {
        let mut tree = json!({
            "name": "Text",
            "autoRename": true,
            "textTracking": 0.0,
            "fontSize": 12.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("autoRename").is_none());
        assert!(tree.get("textTracking").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(12.0));
    }

    #[test]
    fn test_remove_text_decoration_skip_ink() {
        let mut tree = json!({
            "name": "Text",
            "textDecorationSkipInk": true,
            "visible": true
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("textDecorationSkipInk").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_all_text_metadata() {
        let mut tree = json!({
            "name": "ComplexText",
            "textBidiVersion": 1,
            "textExplicitLayoutVersion": 1,
            "textUserLayoutVersion": 5,
            "textDecorationSkipInk": true,
            "fontVariantCommonLigatures": true,
            "fontVariantContextualLigatures": true,
            "fontVariations": [],
            "fontVersion": "",
            "emojiImageSet": {"__enum__": "EmojiImageSet", "value": "APPLE"},
            "autoRename": true,
            "textTracking": 0.0,
            "fontSize": 128.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        // All metadata fields removed
        assert!(tree.get("textBidiVersion").is_none());
        assert!(tree.get("textExplicitLayoutVersion").is_none());
        assert!(tree.get("textUserLayoutVersion").is_none());
        assert!(tree.get("textDecorationSkipInk").is_none());
        assert!(tree.get("fontVariantCommonLigatures").is_none());
        assert!(tree.get("fontVariantContextualLigatures").is_none());
        assert!(tree.get("fontVariations").is_none());
        assert!(tree.get("fontVersion").is_none());
        assert!(tree.get("emojiImageSet").is_none());
        assert!(tree.get("autoRename").is_none());
        assert!(tree.get("textTracking").is_none());

        // Other fields preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("ComplexText"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(128.0));
    }

    #[test]
    fn test_nested_text_metadata() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "textBidiVersion": 1,
                    "autoRename": true
                },
                {
                    "name": "Child2",
                    "children": [
                        {
                            "name": "DeepChild",
                            "textUserLayoutVersion": 5,
                            "textTracking": 0.0
                        }
                    ]
                }
            ]
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        // Check first nested text
        assert!(tree["children"][0].get("textBidiVersion").is_none());
        assert!(tree["children"][0].get("autoRename").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child1")
        );

        // Check deeply nested text
        let deep_child = &tree["children"][1]["children"][0];
        assert!(deep_child.get("textUserLayoutVersion").is_none());
        assert!(deep_child.get("textTracking").is_none());
        assert_eq!(deep_child.get("name").unwrap().as_str(), Some("DeepChild"));
    }

    #[test]
    fn test_no_text_metadata() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "visible": true
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        // Tree without text metadata should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_preserves_important_text_fields() {
        let mut tree = json!({
            "name": "Hello",
            "autoRename": true,
            "textTracking": 0.0,
            "fontName": {
                "family": "Inter",
                "style": "Regular"
            },
            "fontSize": 128.0,
            "textData": {
                "characters": "Hello"
            }
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        // Metadata removed
        assert!(tree.get("autoRename").is_none());
        assert!(tree.get("textTracking").is_none());

        // Important fields preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Hello"));
        assert!(tree.get("fontName").is_some());
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(128.0));
        assert!(tree.get("textData").is_some());
        assert_eq!(
            tree["textData"]["characters"].as_str(),
            Some("Hello")
        );
    }

    #[test]
    fn test_remove_text_align_vertical() {
        let mut tree = json!({
            "name": "Text",
            "textAlignVertical": "TOP",
            "fontSize": 16.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("textAlignVertical").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_remove_text_auto_resize() {
        let mut tree = json!({
            "name": "Text",
            "textAutoResize": "WIDTH_AND_HEIGHT",
            "fontSize": 14.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("textAutoResize").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }
}
