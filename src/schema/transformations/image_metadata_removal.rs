use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove image metadata fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes image-related metadata:
/// - "thumbHash" - Thumbnail hash array
/// - "animationFrame" - Animation frame number
/// - "imageShouldColorManage" - Color management flag
/// - "imageScaleMode" - Image scaling mode
/// - "originalImageWidth" - Original image width
/// - "originalImageHeight" - Original image height
/// - "altText" - Alternative text for image
/// - "imageThumbnail" - Thumbnail image (duplicate of image field)
/// - "rotation" - Image rotation (when inside paint objects)
/// - "scale" - Image scale (when inside paint objects)
///
/// These fields contain image metadata that is not essential for basic
/// HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all image metadata fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_image_metadata_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Image",
///     "thumbHash": [],
///     "animationFrame": 0,
///     "imageShouldColorManage": true,
///     "imageScaleMode": {
///         "__enum__": "ImageScaleMode",
///         "value": "FILL"
///     },
///     "visible": true
/// });
/// remove_image_metadata_fields(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_image_metadata_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove image metadata fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove image metadata fields if they exist
            map.remove("thumbHash");
            map.remove("animationFrame");
            map.remove("imageShouldColorManage");
            map.remove("imageScaleMode");
            map.remove("originalImageWidth");
            map.remove("originalImageHeight");
            map.remove("altText");
            map.remove("imageThumbnail");

            // Check if this is a paint object with image properties
            // (rotation and scale should only be removed in certain contexts)
            if map.contains_key("type") {
                if let Some(type_val) = map.get("type") {
                    if let Some(type_obj) = type_val.as_object() {
                        if let Some(value_str) = type_obj.get("value").and_then(|v| v.as_str()) {
                            if value_str == "IMAGE" {
                                // This is an image paint object, remove rotation and scale
                                map.remove("rotation");
                                map.remove("scale");
                            }
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_thumb_hash() {
        let mut tree = json!({
            "name": "Image",
            "thumbHash": [],
            "visible": true
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("thumbHash").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_animation_frame() {
        let mut tree = json!({
            "name": "Image",
            "animationFrame": 0,
            "opacity": 1.0
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("animationFrame").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_remove_color_manage_flag() {
        let mut tree = json!({
            "name": "Image",
            "imageShouldColorManage": true,
            "visible": true
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("imageShouldColorManage").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_image_scale_mode() {
        let mut tree = json!({
            "name": "Image",
            "imageScaleMode": {
                "__enum__": "ImageScaleMode",
                "value": "FILL"
            },
            "opacity": 1.0
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("imageScaleMode").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_remove_original_dimensions() {
        let mut tree = json!({
            "name": "Image",
            "originalImageWidth": 300,
            "originalImageHeight": 300,
            "visible": true
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("originalImageWidth").is_none());
        assert!(tree.get("originalImageHeight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_alt_text() {
        let mut tree = json!({
            "name": "Image",
            "altText": "",
            "visible": true
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("altText").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_image_paint_rotation_scale() {
        let mut tree = json!({
            "type": {
                "__enum__": "PaintType",
                "value": "IMAGE"
            },
            "rotation": 0.0,
            "scale": 0.5,
            "opacity": 1.0
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("rotation").is_none());
        assert!(tree.get("scale").is_none());
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_preserve_non_image_rotation_scale() {
        let mut tree = json!({
            "name": "Frame",
            "rotation": 45.0,
            "scale": 2.0,
            "type": "FRAME"
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        // rotation and scale should be preserved for non-image objects
        assert_eq!(tree.get("rotation").unwrap().as_f64(), Some(45.0));
        assert_eq!(tree.get("scale").unwrap().as_f64(), Some(2.0));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_remove_all_image_metadata() {
        let mut tree = json!({
            "name": "ComplexImage",
            "thumbHash": [],
            "animationFrame": 0,
            "imageShouldColorManage": true,
            "imageScaleMode": {
                "__enum__": "ImageScaleMode",
                "value": "FILL"
            },
            "originalImageWidth": 300,
            "originalImageHeight": 300,
            "altText": "",
            "type": {
                "__enum__": "PaintType",
                "value": "IMAGE"
            },
            "rotation": 0.0,
            "scale": 0.5,
            "opacity": 1.0
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        // All metadata fields removed
        assert!(tree.get("thumbHash").is_none());
        assert!(tree.get("animationFrame").is_none());
        assert!(tree.get("imageShouldColorManage").is_none());
        assert!(tree.get("imageScaleMode").is_none());
        assert!(tree.get("originalImageWidth").is_none());
        assert!(tree.get("originalImageHeight").is_none());
        assert!(tree.get("altText").is_none());
        assert!(tree.get("rotation").is_none());
        assert!(tree.get("scale").is_none());

        // Other fields preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("ComplexImage"));
        assert!(tree.get("type").is_some());
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_nested_image_metadata() {
        let mut tree = json!({
            "name": "Root",
            "fillPaints": [
                {
                    "type": {
                        "__enum__": "PaintType",
                        "value": "IMAGE"
                    },
                    "thumbHash": [],
                    "animationFrame": 0,
                    "rotation": 0.0,
                    "scale": 0.5,
                    "opacity": 1.0
                }
            ]
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        // Check nested image paint
        let paint = &tree["fillPaints"][0];
        assert!(paint.get("thumbHash").is_none());
        assert!(paint.get("animationFrame").is_none());
        assert!(paint.get("rotation").is_none());
        assert!(paint.get("scale").is_none());
        assert!(paint.get("type").is_some());
        assert_eq!(paint.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_no_image_metadata() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "visible": true
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        // Tree without image metadata should be unchanged
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_image_thumbnail() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "IMAGE",
                    "image": {
                        "filename": "images/abc123",
                        "name": "Photo"
                    },
                    "imageThumbnail": {
                        "filename": "images/abc123",
                        "name": "Photo"
                    }
                }
            ]
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        let paint = &tree["fillPaints"][0];
        // imageThumbnail should be removed (duplicate of image)
        assert!(paint.get("imageThumbnail").is_none());
        // image should be preserved
        assert!(paint.get("image").is_some());
    }

    #[test]
    fn test_remove_image_thumbnail_nested() {
        let mut tree = json!({
            "children": [
                {
                    "fillPaints": [
                        {
                            "image": {"filename": "images/abc"},
                            "imageThumbnail": {"filename": "images/abc"}
                        }
                    ]
                }
            ]
        });

        remove_image_metadata_fields(&mut tree).unwrap();

        let paint = &tree["children"][0]["fillPaints"][0];
        assert!(paint.get("imageThumbnail").is_none());
        assert!(paint.get("image").is_some());
    }
}
