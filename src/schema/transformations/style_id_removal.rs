use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove style ID reference fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes Figma shared style references:
/// - "styleIdForFill" - Reference to fill paint style
/// - "styleIdForText" - Reference to text style
/// - "styleIdForStrokeFill" - Reference to stroke paint style
///
/// These fields reference Figma's shared style library. The actual style values
/// are already inlined in the node properties, so these references are not needed
/// for HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all style ID fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_style_ids;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Text",
///     "fillPaints": [{"color": "#ffffff", "type": "SOLID"}],
///     "styleIdForFill": {
///         "assetRef": {
///             "key": "abc123",
///             "version": "1:101"
///         }
///     },
///     "fontSize": 14.0
/// });
/// remove_style_ids(&mut tree).unwrap();
/// // tree now has "name", "fillPaints", and "fontSize" fields
/// ```
pub fn remove_style_ids(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove style ID fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove all style ID fields if they exist
            map.remove("styleIdForFill");
            map.remove("styleIdForText");
            map.remove("styleIdForStrokeFill");

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
    fn test_remove_style_id_for_fill() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillPaints": [{"color": "#ff0000", "type": "SOLID"}],
            "styleIdForFill": {
                "assetRef": {
                    "key": "abc123",
                    "version": "1:77"
                }
            },
            "visible": true
        });

        remove_style_ids(&mut tree).unwrap();

        assert!(tree.get("styleIdForFill").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert!(tree.get("fillPaints").is_some());
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_style_id_for_text() {
        let mut tree = json!({
            "name": "TextNode",
            "fontSize": 14.0,
            "styleIdForText": {
                "assetRef": {
                    "key": "def456",
                    "version": "1:161"
                }
            },
            "styleIdForFill": {
                "assetRef": {
                    "key": "ghi789",
                    "version": "1:101"
                }
            }
        });

        remove_style_ids(&mut tree).unwrap();

        assert!(tree.get("styleIdForText").is_none());
        assert!(tree.get("styleIdForFill").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("TextNode"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_remove_style_id_for_stroke_fill() {
        let mut tree = json!({
            "name": "Shape",
            "strokePaints": [{"color": "#000000", "type": "SOLID"}],
            "styleIdForStrokeFill": {
                "assetRef": {
                    "key": "xyz000",
                    "version": "1:83"
                }
            }
        });

        remove_style_ids(&mut tree).unwrap();

        assert!(tree.get("styleIdForStrokeFill").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert!(tree.get("strokePaints").is_some());
    }

    #[test]
    fn test_remove_all_style_ids() {
        let mut tree = json!({
            "name": "StyledNode",
            "styleIdForFill": {"assetRef": {"key": "a", "version": "1:1"}},
            "styleIdForText": {"assetRef": {"key": "b", "version": "1:2"}},
            "styleIdForStrokeFill": {"assetRef": {"key": "c", "version": "1:3"}},
            "visible": true
        });

        remove_style_ids(&mut tree).unwrap();

        assert!(tree.get("styleIdForFill").is_none());
        assert!(tree.get("styleIdForText").is_none());
        assert!(tree.get("styleIdForStrokeFill").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("StyledNode"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_style_ids_nested() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "styleIdForFill": {"assetRef": {"key": "x", "version": "1:1"}}
                },
                {
                    "name": "Child2",
                    "styleIdForText": {"assetRef": {"key": "y", "version": "1:2"}}
                }
            ]
        });

        remove_style_ids(&mut tree).unwrap();

        assert!(tree["children"][0].get("styleIdForFill").is_none());
        assert!(tree["children"][1].get("styleIdForText").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_remove_style_ids_deeply_nested() {
        let mut tree = json!({
            "document": {
                "styleIdForFill": {"assetRef": {"key": "a", "version": "1:1"}},
                "children": [
                    {
                        "styleIdForText": {"assetRef": {"key": "b", "version": "1:2"}},
                        "children": [
                            {
                                "styleIdForStrokeFill": {"assetRef": {"key": "c", "version": "1:3"}},
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_style_ids(&mut tree).unwrap();

        assert!(tree["document"].get("styleIdForFill").is_none());
        assert!(tree["document"]["children"][0].get("styleIdForText").is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("styleIdForStrokeFill")
            .is_none());
    }

    #[test]
    fn test_remove_style_ids_missing() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "visible": true
        });

        remove_style_ids(&mut tree).unwrap();

        assert!(tree.get("styleIdForFill").is_none());
        assert!(tree.get("styleIdForText").is_none());
        assert!(tree.get("styleIdForStrokeFill").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_remove_style_ids_preserves_actual_styles() {
        let mut tree = json!({
            "name": "Text",
            "fillPaints": [{"color": "#ffffff", "type": "SOLID"}],
            "fontSize": 14.0,
            "fontName": {"family": "Inter", "style": "Medium"},
            "styleIdForFill": {"assetRef": {"key": "a", "version": "1:1"}},
            "styleIdForText": {"assetRef": {"key": "b", "version": "1:2"}}
        });

        remove_style_ids(&mut tree).unwrap();

        // Style IDs removed
        assert!(tree.get("styleIdForFill").is_none());
        assert!(tree.get("styleIdForText").is_none());

        // Actual style values preserved
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
        assert!(tree.get("fillPaints").is_some());
        assert!(tree.get("fontName").is_some());
    }

    #[test]
    fn test_remove_style_ids_empty_style_id() {
        let mut tree = json!({
            "name": "Node",
            "styleIdForFill": {},
            "visible": true
        });

        remove_style_ids(&mut tree).unwrap();

        assert!(tree.get("styleIdForFill").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_style_ids_in_symbol_overrides() {
        let mut tree = json!({
            "symbolData": {
                "symbolOverrides": [
                    {
                        "styleIdForFill": {"assetRef": {"key": "a", "version": "1:1"}},
                        "fillPaints": [{"color": "#ff0000", "type": "SOLID"}]
                    },
                    {
                        "styleIdForText": {"assetRef": {"key": "b", "version": "1:2"}},
                        "fontSize": 12.0
                    }
                ]
            }
        });

        remove_style_ids(&mut tree).unwrap();

        assert!(tree["symbolData"]["symbolOverrides"][0]
            .get("styleIdForFill")
            .is_none());
        assert!(tree["symbolData"]["symbolOverrides"][1]
            .get("styleIdForText")
            .is_none());
        assert!(tree["symbolData"]["symbolOverrides"][0]
            .get("fillPaints")
            .is_some());
        assert_eq!(
            tree["symbolData"]["symbolOverrides"][1]["fontSize"].as_f64(),
            Some(12.0)
        );
    }
}
