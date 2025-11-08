use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove exportSettings fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes all "exportSettings" fields.
/// These fields contain Figma asset export configurations (SVG, PNG settings, etc.)
/// that are not needed for HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all exportSettings fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_export_settings;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Icon",
///     "exportSettings": [
///         {
///             "colorProfile": "DOCUMENT",
///             "constraint": {"type": "CONTENT_SCALE", "value": 1.0},
///             "imageType": "SVG"
///         }
///     ],
///     "visible": true
/// });
/// remove_export_settings(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_export_settings(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove exportSettings fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the "exportSettings" field if it exists
            map.remove("exportSettings");

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
    fn test_remove_export_settings_svg() {
        let mut tree = json!({
            "name": "Icon",
            "exportSettings": [
                {
                    "colorProfile": "DOCUMENT",
                    "constraint": {"type": "CONTENT_SCALE", "value": 1.0},
                    "contentsOnly": true,
                    "imageType": "SVG",
                    "svgOutlineText": true
                }
            ],
            "visible": true
        });

        remove_export_settings(&mut tree).unwrap();

        assert!(tree.get("exportSettings").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Icon"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_export_settings_png() {
        let mut tree = json!({
            "name": "Image",
            "exportSettings": [
                {
                    "colorProfile": "DOCUMENT",
                    "constraint": {"type": "CONTENT_SCALE", "value": 1.0},
                    "imageType": "PNG",
                    "useAbsoluteBounds": false
                }
            ]
        });

        remove_export_settings(&mut tree).unwrap();

        assert!(tree.get("exportSettings").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
    }

    #[test]
    fn test_remove_export_settings_multiple() {
        let mut tree = json!({
            "name": "Asset",
            "exportSettings": [
                {
                    "imageType": "SVG",
                    "suffix": "@svg"
                },
                {
                    "imageType": "PNG",
                    "constraint": {"type": "SCALE", "value": 2.0},
                    "suffix": "@2x"
                },
                {
                    "imageType": "PNG",
                    "constraint": {"type": "SCALE", "value": 3.0},
                    "suffix": "@3x"
                }
            ]
        });

        remove_export_settings(&mut tree).unwrap();

        assert!(tree.get("exportSettings").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Asset"));
    }

    #[test]
    fn test_remove_export_settings_nested() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Icon1",
                    "exportSettings": [
                        {"imageType": "SVG"}
                    ]
                },
                {
                    "name": "Icon2",
                    "exportSettings": [
                        {"imageType": "PNG"}
                    ]
                }
            ]
        });

        remove_export_settings(&mut tree).unwrap();

        assert!(tree["children"][0].get("exportSettings").is_none());
        assert!(tree["children"][1].get("exportSettings").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Icon1"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Icon2"));
    }

    #[test]
    fn test_remove_export_settings_deeply_nested() {
        let mut tree = json!({
            "document": {
                "exportSettings": [{"imageType": "SVG"}],
                "children": [
                    {
                        "exportSettings": [{"imageType": "PNG"}],
                        "children": [
                            {
                                "exportSettings": [{"imageType": "JPG"}],
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_export_settings(&mut tree).unwrap();

        assert!(tree["document"].get("exportSettings").is_none());
        assert!(tree["document"]["children"][0].get("exportSettings").is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("exportSettings")
            .is_none());
    }

    #[test]
    fn test_remove_export_settings_missing() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "visible": true
        });

        remove_export_settings(&mut tree).unwrap();

        assert!(tree.get("exportSettings").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_remove_export_settings_empty_array() {
        let mut tree = json!({
            "name": "Node",
            "exportSettings": [],
            "visible": true
        });

        remove_export_settings(&mut tree).unwrap();

        assert!(tree.get("exportSettings").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_export_settings_preserves_other_fields() {
        let mut tree = json!({
            "name": "Icon",
            "exportSettings": [
                {
                    "imageType": "SVG",
                    "svgOutlineText": true,
                    "svgIDMode": "IF_NEEDED"
                }
            ],
            "size": {"x": 20, "y": 20},
            "fillPaints": [{"color": "#4d81ee", "type": "SOLID"}],
            "visible": true
        });

        remove_export_settings(&mut tree).unwrap();

        assert!(tree.get("exportSettings").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Icon"));
        assert!(tree.get("size").is_some());
        assert!(tree.get("fillPaints").is_some());
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_export_settings_in_symbol_overrides() {
        let mut tree = json!({
            "symbolData": {
                "symbolOverrides": [
                    {
                        "exportSettings": [
                            {"imageType": "SVG"}
                        ],
                        "name": "icon/ai"
                    }
                ]
            }
        });

        remove_export_settings(&mut tree).unwrap();

        assert!(tree["symbolData"]["symbolOverrides"][0]
            .get("exportSettings")
            .is_none());
        assert_eq!(
            tree["symbolData"]["symbolOverrides"][0]["name"].as_str(),
            Some("icon/ai")
        );
    }

    #[test]
    fn test_remove_export_settings_complex_config() {
        let mut tree = json!({
            "name": "ComplexAsset",
            "exportSettings": [
                {
                    "colorProfile": "DOCUMENT",
                    "constraint": {"type": "CONTENT_SCALE", "value": 1.0},
                    "contentsOnly": true,
                    "imageType": "SVG",
                    "suffix": "",
                    "svgDataName": false,
                    "svgForceStrokeMasks": false,
                    "svgIDMode": "IF_NEEDED",
                    "svgOutlineText": true,
                    "useAbsoluteBounds": false,
                    "useBicubicSampler": true
                }
            ]
        });

        remove_export_settings(&mut tree).unwrap();

        assert!(tree.get("exportSettings").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("ComplexAsset"));
    }
}
