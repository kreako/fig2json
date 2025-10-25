pub mod parser;
pub mod substitution;

use crate::error::Result;
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value as JsonValue;

// Re-export commonly used items
pub use parser::{parse_blob, parse_commands, parse_vector_network};
pub use substitution::substitute_blobs;

/// Process blobs array by encoding binary data to base64
///
/// Takes the blobs array from decoded Kiwi data and converts any binary
/// byte arrays to base64-encoded strings for JSON compatibility.
///
/// # Arguments
/// * `blobs` - Array of blob objects from decoded Kiwi data
///
/// # Returns
/// * `Ok(JsonValue)` - Processed blobs array with base64-encoded data
/// * `Err(FigError)` - If blob processing fails
///
/// # Examples
/// ```no_run
/// use fig2json::blobs::process_blobs;
/// use serde_json::json;
///
/// let blobs = vec![/* blob objects */];
/// let processed = process_blobs(blobs).unwrap();
/// ```
pub fn process_blobs(blobs: Vec<JsonValue>) -> Result<JsonValue> {
    let mut processed_blobs = Vec::new();

    for blob in blobs {
        let mut processed_blob = blob.clone();

        // If blob has a bytes field with an array, encode it to base64
        if let Some(obj) = processed_blob.as_object_mut() {
            if let Some(bytes_value) = obj.get("bytes") {
                if let Some(bytes_array) = bytes_value.as_array() {
                    // Convert JSON array of numbers to byte vector
                    let bytes: Vec<u8> = bytes_array
                        .iter()
                        .filter_map(|v| v.as_u64().map(|n| n as u8))
                        .collect();

                    // Encode to base64
                    let base64_string = general_purpose::STANDARD.encode(&bytes);

                    // Replace bytes array with base64 string
                    obj.insert("bytes".to_string(), JsonValue::String(base64_string));
                }
            }
        }

        processed_blobs.push(processed_blob);
    }

    Ok(JsonValue::Array(processed_blobs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_process_blobs_with_bytes() {
        let blobs = vec![json!({
            "id": 1,
            "bytes": [72, 101, 108, 108, 111]  // "Hello" in ASCII
        })];

        let processed = process_blobs(blobs).unwrap();
        let blobs_array = processed.as_array().unwrap();

        assert_eq!(blobs_array.len(), 1);
        let blob = &blobs_array[0];

        // Check that bytes field is now a base64 string
        let bytes_value = blob.get("bytes").unwrap();
        assert!(bytes_value.is_string());
        assert_eq!(bytes_value.as_str().unwrap(), "SGVsbG8=");  // "Hello" in base64
    }

    #[test]
    fn test_process_blobs_without_bytes() {
        let blobs = vec![json!({
            "id": 1,
            "type": "IMAGE"
        })];

        let processed = process_blobs(blobs).unwrap();
        let blobs_array = processed.as_array().unwrap();

        assert_eq!(blobs_array.len(), 1);
        assert_eq!(blobs_array[0].get("id").unwrap(), 1);
    }

    #[test]
    fn test_process_empty_blobs() {
        let blobs = vec![];
        let processed = process_blobs(blobs).unwrap();
        let blobs_array = processed.as_array().unwrap();
        assert_eq!(blobs_array.len(), 0);
    }

    #[test]
    fn test_process_multiple_blobs() {
        let blobs = vec![
            json!({
                "id": 1,
                "bytes": [65, 66, 67]  // "ABC"
            }),
            json!({
                "id": 2,
                "bytes": [88, 89, 90]  // "XYZ"
            }),
        ];

        let processed = process_blobs(blobs).unwrap();
        let blobs_array = processed.as_array().unwrap();

        assert_eq!(blobs_array.len(), 2);
        assert_eq!(blobs_array[0].get("bytes").unwrap().as_str().unwrap(), "QUJD");  // "ABC" in base64
        assert_eq!(blobs_array[1].get("bytes").unwrap().as_str().unwrap(), "WFla");  // "XYZ" in base64
    }
}
