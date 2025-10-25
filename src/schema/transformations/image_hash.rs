use crate::error::Result;
use serde_json::Value as JsonValue;

/// Transform image hash arrays to filename strings
///
/// Recursively traverses the JSON tree and transforms objects in "image" and
/// "imageThumbnail" fields by:
/// - Converting "hash" array of integers to hex-encoded "filename" string
/// - Removing the "hash" field
/// - Preserving all other fields (including "name")
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully transformed all image hashes
///
/// # Examples
/// ```no_run
/// use fig2json::schema::transform_image_hashes;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "image": {
///         "hash": [96, 73, 161, 122],
///         "name": "Amazon-beast"
///     }
/// });
/// transform_image_hashes(&mut tree).unwrap();
/// // tree now has "image": {"filename": "images/6049a17a", "name": "Amazon-beast"}
/// ```
pub fn transform_image_hashes(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively transform image hashes in a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // First, check if this object is in an "image" or "imageThumbnail" field
            // We need to transform any such fields we find
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "image" || key == "imageThumbnail" {
                    // This field might need transformation
                    if let Some(image_obj) = map.get_mut(&key) {
                        if let Some(obj) = image_obj.as_object_mut() {
                            // Check if it has a "hash" field
                            if let Some(hash_value) = obj.get("hash") {
                                if let Some(hash_array) = hash_value.as_array() {
                                    // Convert hash array to filename
                                    if let Some(filename) = hash_to_filename(hash_array) {
                                        // Remove hash field
                                        obj.remove("hash");
                                        // Add filename field
                                        obj.insert("filename".to_string(), JsonValue::String(filename));
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

/// Convert a hash array of integers to a filename string
///
/// Converts each integer to its 2-digit hex representation and concatenates
/// them with "images/" prefix.
///
/// # Arguments
/// * `hash` - Array of integers representing the hash
///
/// # Returns
/// * `Some(String)` - The filename string (e.g., "images/6049a17a...")
/// * `None` - If any element is not a valid u8 integer
fn hash_to_filename(hash: &[JsonValue]) -> Option<String> {
    let mut hex_string = String::with_capacity(hash.len() * 2);

    for value in hash {
        if let Some(num) = value.as_u64() {
            if num <= 255 {
                // Format as 2-digit lowercase hex
                hex_string.push_str(&format!("{:02x}", num));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(format!("images/{}", hex_string))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hash_to_filename() {
        let hash = vec![
            JsonValue::from(96),
            JsonValue::from(73),
            JsonValue::from(161),
            JsonValue::from(122),
        ];

        let filename = hash_to_filename(&hash).unwrap();
        assert_eq!(filename, "images/6049a17a");
    }

    #[test]
    fn test_hash_to_filename_full() {
        let hash = vec![
            JsonValue::from(96), JsonValue::from(73), JsonValue::from(161), JsonValue::from(122),
            JsonValue::from(132), JsonValue::from(131), JsonValue::from(226), JsonValue::from(80),
            JsonValue::from(226), JsonValue::from(150), JsonValue::from(78), JsonValue::from(100),
            JsonValue::from(84), JsonValue::from(218), JsonValue::from(142), JsonValue::from(231),
            JsonValue::from(161), JsonValue::from(69), JsonValue::from(66), JsonValue::from(133),
        ];

        let filename = hash_to_filename(&hash).unwrap();
        assert_eq!(filename, "images/6049a17a8483e250e2964e6454da8ee7a1454285");
    }

    #[test]
    fn test_hash_to_filename_invalid() {
        let hash = vec![JsonValue::from(256)]; // Out of u8 range
        assert!(hash_to_filename(&hash).is_none());
    }

    #[test]
    fn test_transform_image_field() {
        let mut tree = json!({
            "name": "Rectangle",
            "image": {
                "hash": [96, 73, 161, 122],
                "name": "Amazon-beast"
            }
        });

        transform_image_hashes(&mut tree).unwrap();

        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049a17a"));
        assert_eq!(image.get("name").unwrap().as_str(), Some("Amazon-beast"));
    }

    #[test]
    fn test_transform_image_thumbnail_field() {
        let mut tree = json!({
            "name": "Rectangle",
            "imageThumbnail": {
                "hash": [96, 73, 161, 122, 132, 131],
                "name": "Test-Image"
            }
        });

        transform_image_hashes(&mut tree).unwrap();

        let thumbnail = tree.get("imageThumbnail").unwrap();
        assert!(thumbnail.get("hash").is_none());
        assert_eq!(thumbnail.get("filename").unwrap().as_str(), Some("images/6049a17a8483"));
        assert_eq!(thumbnail.get("name").unwrap().as_str(), Some("Test-Image"));
    }

    #[test]
    fn test_transform_nested_objects() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "image": {
                        "hash": [96, 73],
                        "name": "Image1"
                    }
                },
                {
                    "name": "Child2",
                    "fills": [
                        {
                            "image": {
                                "hash": [161, 122],
                                "name": "Image2"
                            }
                        }
                    ]
                }
            ]
        });

        transform_image_hashes(&mut tree).unwrap();

        // Check first nested image
        let child1_image = &tree["children"][0]["image"];
        assert!(child1_image.get("hash").is_none());
        assert_eq!(child1_image.get("filename").unwrap().as_str(), Some("images/6049"));

        // Check deeply nested image
        let child2_image = &tree["children"][1]["fills"][0]["image"];
        assert!(child2_image.get("hash").is_none());
        assert_eq!(child2_image.get("filename").unwrap().as_str(), Some("images/a17a"));
    }

    #[test]
    fn test_transform_preserves_other_fields() {
        let mut tree = json!({
            "name": "Rectangle",
            "visible": true,
            "image": {
                "hash": [96, 73, 161, 122],
                "name": "Amazon-beast",
                "width": 100,
                "height": 200
            },
            "x": 10,
            "y": 20
        });

        transform_image_hashes(&mut tree).unwrap();

        // Check that non-image fields are preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));

        // Check that image object preserves all fields except hash
        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049a17a"));
        assert_eq!(image.get("name").unwrap().as_str(), Some("Amazon-beast"));
        assert_eq!(image.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(image.get("height").unwrap().as_i64(), Some(200));
    }

    #[test]
    fn test_transform_no_hash_field() {
        let mut tree = json!({
            "name": "Rectangle",
            "image": {
                "name": "Amazon-beast",
                "url": "https://example.com/image.png"
            }
        });

        transform_image_hashes(&mut tree).unwrap();

        // Image without hash should be unchanged
        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert!(image.get("filename").is_none());
        assert_eq!(image.get("name").unwrap().as_str(), Some("Amazon-beast"));
        assert_eq!(image.get("url").unwrap().as_str(), Some("https://example.com/image.png"));
    }

    #[test]
    fn test_transform_both_image_and_thumbnail() {
        let mut tree = json!({
            "name": "Rectangle",
            "image": {
                "hash": [96, 73],
                "name": "Main-Image"
            },
            "imageThumbnail": {
                "hash": [161, 122],
                "name": "Thumbnail"
            }
        });

        transform_image_hashes(&mut tree).unwrap();

        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049"));
        assert_eq!(image.get("name").unwrap().as_str(), Some("Main-Image"));

        let thumbnail = tree.get("imageThumbnail").unwrap();
        assert!(thumbnail.get("hash").is_none());
        assert_eq!(thumbnail.get("filename").unwrap().as_str(), Some("images/a17a"));
        assert_eq!(thumbnail.get("name").unwrap().as_str(), Some("Thumbnail"));
    }

    #[test]
    fn test_transform_ignores_other_hash_fields() {
        let mut tree = json!({
            "name": "Node",
            "metadata": {
                "hash": [1, 2, 3, 4],
                "type": "checksum"
            },
            "image": {
                "hash": [96, 73],
                "name": "Real-Image"
            }
        });

        transform_image_hashes(&mut tree).unwrap();

        // metadata.hash should remain unchanged (not in "image" or "imageThumbnail" field)
        let metadata = tree.get("metadata").unwrap();
        assert!(metadata.get("hash").is_some());
        assert!(metadata.get("filename").is_none());

        // image.hash should be transformed
        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049"));
    }
}
