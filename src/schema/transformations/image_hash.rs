use crate::error::Result;
use serde_json::Value as JsonValue;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Transform image hash arrays to filename strings with extensions
///
/// Recursively traverses the JSON tree and transforms objects in "image" and
/// "imageThumbnail" fields by:
/// - Converting "hash" array of integers to hex-encoded "filename" string
/// - Detecting image format from file header (PNG, JPEG, WebP, GIF, SVG)
/// - Renaming physical files to include the appropriate extension
/// - Updating the filename in JSON to include the extension
/// - Removing the "hash" field
/// - Preserving all other fields (including "name")
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
/// * `base_dir` - Base directory where image files are located (relative to output JSON)
///
/// # Returns
/// * `Ok(())` - Successfully transformed all image hashes
///
/// # Examples
/// ```no_run
/// use fig2json::schema::transform_image_hashes;
/// use serde_json::json;
/// use std::path::Path;
///
/// let mut tree = json!({
///     "image": {
///         "hash": [96, 73, 161, 122],
///         "name": "Amazon-beast"
///     }
/// });
/// transform_image_hashes(&mut tree, Path::new("/output/dir")).unwrap();
/// // tree now has "image": {"filename": "images/6049a17a.jpg", "name": "Amazon-beast"}
/// ```
pub fn transform_image_hashes(tree: &mut JsonValue, base_dir: &Path) -> Result<()> {
    transform_recursive(tree, base_dir)
}

/// Recursively transform image hashes in a JSON value
fn transform_recursive(value: &mut JsonValue, base_dir: &Path) -> Result<()> {
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
                                    if let Some(mut filename) = hash_to_filename(hash_array) {
                                        // Try to detect format and rename physical file
                                        let file_path = base_dir.join(&filename);

                                        if let Some(extension) = detect_image_format(&file_path) {
                                            // Rename physical file with extension
                                            let new_filename = format!("{}{}", filename, extension);
                                            let new_file_path = base_dir.join(&new_filename);

                                            // Attempt to rename the file
                                            // If it fails, we'll still update the JSON with the extension
                                            // (user may have already renamed files, or file may not exist yet)
                                            let _ = fs::rename(&file_path, &new_file_path);

                                            // Update filename to include extension
                                            filename = new_filename;
                                        }

                                        // Remove hash field
                                        obj.remove("hash");
                                        // Add filename field (with or without extension)
                                        obj.insert("filename".to_string(), JsonValue::String(filename));
                                    }
                                }
                            }
                        }
                    }
                }

                // Recurse into the value regardless
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val, base_dir)?;
                }
            }
        }
        JsonValue::Array(arr) => {
            // Recurse into array elements
            for val in arr.iter_mut() {
                transform_recursive(val, base_dir)?;
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

/// Detect image format from file header (magic bytes)
///
/// Reads the first few bytes of a file to identify the image format.
///
/// # Arguments
/// * `file_path` - Path to the image file
///
/// # Returns
/// * `Some(String)` - The file extension (e.g., ".png", ".jpg", ".webp", ".gif", ".svg")
/// * `None` - If format cannot be detected or file cannot be read
fn detect_image_format(file_path: &Path) -> Option<String> {
    // Read first 256 bytes for format detection
    let mut file = fs::File::open(file_path).ok()?;
    let mut buffer = vec![0u8; 256];
    let bytes_read = file.read(&mut buffer).ok()?;

    if bytes_read < 4 {
        return None;
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if bytes_read >= 8 && buffer[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some(".png".to_string());
    }

    // JPEG: FF D8 FF
    if bytes_read >= 3 && buffer[0..3] == [0xFF, 0xD8, 0xFF] {
        return Some(".jpg".to_string());
    }

    // GIF: 47 49 46 38 (GIF8)
    if bytes_read >= 4 && buffer[0..4] == [0x47, 0x49, 0x46, 0x38] {
        return Some(".gif".to_string());
    }

    // WebP: 52 49 46 46 [4 bytes] 57 45 42 50 (RIFF....WEBP)
    if bytes_read >= 12
        && buffer[0..4] == [0x52, 0x49, 0x46, 0x46]
        && buffer[8..12] == [0x57, 0x45, 0x42, 0x50] {
        return Some(".webp".to_string());
    }

    // SVG: Check for XML/SVG markers (text-based)
    if let Ok(text) = std::str::from_utf8(&buffer[..bytes_read]) {
        let text_lower = text.to_lowercase();
        if text_lower.contains("<svg") || (text_lower.contains("<?xml") && text_lower.contains("svg")) {
            return Some(".svg".to_string());
        }
    }

    None
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

        // Use a test path (file won't exist, so no extension will be added)
        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

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

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

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

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

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

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

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

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

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

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

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

        transform_image_hashes(&mut tree, std::path::Path::new(".")).unwrap();

        // metadata.hash should remain unchanged (not in "image" or "imageThumbnail" field)
        let metadata = tree.get("metadata").unwrap();
        assert!(metadata.get("hash").is_some());
        assert!(metadata.get("filename").is_none());

        // image.hash should be transformed
        let image = tree.get("image").unwrap();
        assert!(image.get("hash").is_none());
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049"));
    }

    // Tests for image format detection with actual files

    #[test]
    fn test_detect_png_format() {
        use std::io::Write;

        // Create a temporary directory
        let temp_dir = std::env::temp_dir().join("fig2json_test_png");
        let _ = fs::create_dir_all(&temp_dir);

        // Create a test file with PNG magic bytes
        let test_file = temp_dir.join("images").join("6049a17a");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();
        file.write_all(&[0; 100]).unwrap(); // Add some padding
        drop(file);

        // Test transformation
        let mut tree = json!({
            "image": {
                "hash": [96, 73, 161, 122],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/6049a17a.png"));

        // Verify file was renamed
        assert!(temp_dir.join("images/6049a17a.png").exists());
        assert!(!test_file.exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_jpeg_format() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_jpeg");
        let _ = fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("images").join("a17a6049");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(&[0xFF, 0xD8, 0xFF, 0xE0]).unwrap(); // JPEG magic bytes
        file.write_all(&[0; 100]).unwrap();
        drop(file);

        let mut tree = json!({
            "image": {
                "hash": [161, 122, 96, 73],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/a17a6049.jpg"));

        assert!(temp_dir.join("images/a17a6049.jpg").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_gif_format() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_gif");
        let _ = fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("images").join("12345678");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"GIF89a").unwrap(); // GIF magic bytes
        file.write_all(&[0; 100]).unwrap();
        drop(file);

        let mut tree = json!({
            "image": {
                "hash": [0x12, 0x34, 0x56, 0x78],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/12345678.gif"));

        assert!(temp_dir.join("images/12345678.gif").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_webp_format() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_webp");
        let _ = fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("images").join("abcdef12");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"RIFF").unwrap();
        file.write_all(&[0, 0, 0, 0]).unwrap(); // Size placeholder
        file.write_all(b"WEBP").unwrap();
        file.write_all(&[0; 100]).unwrap();
        drop(file);

        let mut tree = json!({
            "image": {
                "hash": [0xab, 0xcd, 0xef, 0x12],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/abcdef12.webp"));

        assert!(temp_dir.join("images/abcdef12.webp").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_svg_format() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_svg");
        let _ = fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("images").join("87654321");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\">").unwrap();
        file.write_all(&[0; 100]).unwrap();
        drop(file);

        let mut tree = json!({
            "image": {
                "hash": [0x87, 0x65, 0x43, 0x21],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/87654321.svg"));

        assert!(temp_dir.join("images/87654321.svg").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_unknown_format_keeps_no_extension() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_unknown");
        let _ = fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("images").join("deadbeef");
        fs::create_dir_all(test_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"UNKNOWN FORMAT").unwrap(); // Unknown magic bytes
        file.write_all(&[0; 100]).unwrap();
        drop(file);

        let mut tree = json!({
            "image": {
                "hash": [0xde, 0xad, 0xbe, 0xef],
                "name": "Test"
            }
        });

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let image = tree.get("image").unwrap();
        // Unknown format should keep filename without extension
        assert_eq!(image.get("filename").unwrap().as_str(), Some("images/deadbeef"));

        // Original file should still exist (not renamed)
        assert!(test_file.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_multiple_images_different_formats() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("fig2json_test_multi");
        let _ = fs::create_dir_all(&temp_dir);

        // Create PNG file
        let png_file = temp_dir.join("images").join("6049");
        fs::create_dir_all(png_file.parent().unwrap()).unwrap();
        let mut file = fs::File::create(&png_file).unwrap();
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();
        drop(file);

        // Create JPEG file
        let jpg_file = temp_dir.join("images").join("a17a");
        let mut file = fs::File::create(&jpg_file).unwrap();
        file.write_all(&[0xFF, 0xD8, 0xFF, 0xE0]).unwrap();
        drop(file);

        let mut tree = json!({
            "children": [
                {
                    "image": {
                        "hash": [96, 73],
                        "name": "Image1"
                    }
                },
                {
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

        transform_image_hashes(&mut tree, &temp_dir).unwrap();

        let child1_image = &tree["children"][0]["image"];
        assert_eq!(child1_image.get("filename").unwrap().as_str(), Some("images/6049.png"));

        let child2_image = &tree["children"][1]["fills"][0]["image"];
        assert_eq!(child2_image.get("filename").unwrap().as_str(), Some("images/a17a.jpg"));

        assert!(temp_dir.join("images/6049.png").exists());
        assert!(temp_dir.join("images/a17a.jpg").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
