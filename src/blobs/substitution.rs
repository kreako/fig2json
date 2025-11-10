use crate::blobs::parser::parse_blob;
use crate::error::Result;
use serde_json::Value as JsonValue;

/// Substitute blob references in the document tree with parsed blob content
///
/// Recursively traverses the JSON tree and replaces fields ending in "Blob"
/// with their parsed content. For example:
/// - `commandsBlob: 5` → `commands: ["M", 1.0, 2.0, ...]`
/// - `vectorNetworkBlob: 3` → `vectorNetwork: {vertices: [...], ...}`
///
/// The blob index is used to look up the blob in the blobs array, which is then
/// parsed based on the field name (with "Blob" suffix removed).
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
/// * `blobs` - Array of blob objects containing binary data
///
/// # Returns
/// * `Ok(())` - Successfully substituted all blob references
/// * `Err(FigError)` - If blob parsing or access fails
///
/// # Examples
/// ```no_run
/// use fig2json::blobs::substitute_blobs;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "commandsBlob": 0,
///     "children": []
/// });
/// let blobs = vec![json!({"bytes": vec![0u8]})];
/// substitute_blobs(&mut tree, &blobs).unwrap();
/// // tree now has "commands" field instead of "commandsBlob"
/// ```
pub fn substitute_blobs(tree: &mut JsonValue, blobs: &[JsonValue]) -> Result<()> {
    substitute_blobs_recursive(tree, blobs)
}

/// Recursively substitute blob references in a JSON value
fn substitute_blobs_recursive(value: &mut JsonValue, blobs: &[JsonValue]) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Collect blob fields to replace (can't modify map while iterating)
            let mut replacements = Vec::new();

            for (key, val) in map.iter() {
                if key.ends_with("Blob") {
                    if let Some(index) = val.as_u64() {
                        let index = index as usize;
                        if index < blobs.len() {
                            // Parse the blob
                            let blob_type = &key[..key.len() - 4]; // Remove "Blob" suffix
                            if let Some(parsed) = parse_blob(blob_type, &blobs[index])? {
                                replacements.push((key.clone(), blob_type.to_string(), parsed));
                            }
                        }
                    }
                }
            }

            // Apply replacements
            for (old_key, new_key, new_value) in replacements {
                map.remove(&old_key);
                map.insert(new_key, new_value);
            }

            // Recurse into all values
            for val in map.values_mut() {
                substitute_blobs_recursive(val, blobs)?;
            }
        }
        JsonValue::Array(arr) => {
            // Recurse into array elements
            for val in arr.iter_mut() {
                substitute_blobs_recursive(val, blobs)?;
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
    fn test_substitute_commands_blob() {
        // Create a simple commands blob (M 10 20 Z)
        let mut blob_bytes = Vec::new();
        blob_bytes.push(1); // M
        blob_bytes.extend_from_slice(&10.0f32.to_le_bytes());
        blob_bytes.extend_from_slice(&20.0f32.to_le_bytes());
        blob_bytes.push(0); // Z

        let blobs = vec![json!({
            "bytes": blob_bytes
        })];

        let mut tree = json!({
            "name": "Rectangle",
            "commandsBlob": 0
        });

        substitute_blobs(&mut tree, &blobs).unwrap();

        // Check that commandsBlob was replaced with commands
        assert!(tree.get("commandsBlob").is_none());
        assert!(tree.get("commands").is_some());

        let commands = tree.get("commands").unwrap().as_array().unwrap();
        assert_eq!(commands[0].as_str(), Some("M"));
        assert_eq!(commands[1].as_f64(), Some(10.0));
        assert_eq!(commands[2].as_f64(), Some(20.0));
        assert_eq!(commands[3].as_str(), Some("Z"));
    }

    #[test]
    fn test_substitute_vector_network_blob() {
        // Create a simple vector network blob (2 vertices, 1 segment, 0 regions)
        let mut blob_bytes = Vec::new();
        blob_bytes.extend_from_slice(&2u32.to_le_bytes()); // 2 vertices
        blob_bytes.extend_from_slice(&1u32.to_le_bytes()); // 1 segment
        blob_bytes.extend_from_slice(&0u32.to_le_bytes()); // 0 regions

        // Vertex 0
        blob_bytes.extend_from_slice(&0u32.to_le_bytes()); // styleID
        blob_bytes.extend_from_slice(&10.0f32.to_le_bytes()); // x
        blob_bytes.extend_from_slice(&20.0f32.to_le_bytes()); // y

        // Vertex 1
        blob_bytes.extend_from_slice(&0u32.to_le_bytes());
        blob_bytes.extend_from_slice(&30.0f32.to_le_bytes());
        blob_bytes.extend_from_slice(&40.0f32.to_le_bytes());

        // Segment 0
        blob_bytes.extend_from_slice(&0u32.to_le_bytes()); // styleID
        blob_bytes.extend_from_slice(&0u32.to_le_bytes()); // start vertex
        blob_bytes.extend_from_slice(&0.0f32.to_le_bytes()); // start dx
        blob_bytes.extend_from_slice(&0.0f32.to_le_bytes()); // start dy
        blob_bytes.extend_from_slice(&1u32.to_le_bytes()); // end vertex
        blob_bytes.extend_from_slice(&0.0f32.to_le_bytes()); // end dx
        blob_bytes.extend_from_slice(&0.0f32.to_le_bytes()); // end dy

        let blobs = vec![json!({
            "bytes": blob_bytes
        })];

        let mut tree = json!({
            "name": "Vector",
            "vectorNetworkBlob": 0
        });

        substitute_blobs(&mut tree, &blobs).unwrap();

        // Check that vectorNetworkBlob was replaced with vectorNetwork
        assert!(tree.get("vectorNetworkBlob").is_none());
        assert!(tree.get("vectorNetwork").is_some());

        let network = tree.get("vectorNetwork").unwrap();
        assert!(network.get("vertices").is_some());
        assert!(network.get("segments").is_some());
        assert!(network.get("regions").is_some());

        let vertices = network.get("vertices").unwrap().as_array().unwrap();
        assert_eq!(vertices.len(), 2);
    }

    #[test]
    fn test_substitute_nested_tree() {
        // Create a commands blob
        let mut blob_bytes = Vec::new();
        blob_bytes.push(1); // M
        blob_bytes.extend_from_slice(&5.0f32.to_le_bytes());
        blob_bytes.extend_from_slice(&10.0f32.to_le_bytes());
        blob_bytes.push(0); // Z

        let blobs = vec![json!({
            "bytes": blob_bytes
        })];

        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "commandsBlob": 0
                },
                {
                    "name": "Child2",
                    "children": [
                        {
                            "name": "GrandChild",
                            "commandsBlob": 0
                        }
                    ]
                }
            ]
        });

        substitute_blobs(&mut tree, &blobs).unwrap();

        // Check that all commandsBlob references were replaced
        let child1 = &tree["children"][0];
        assert!(child1.get("commandsBlob").is_none());
        assert!(child1.get("commands").is_some());

        let grandchild = &tree["children"][1]["children"][0];
        assert!(grandchild.get("commandsBlob").is_none());
        assert!(grandchild.get("commands").is_some());
    }

    #[test]
    fn test_substitute_multiple_blob_types() {
        // Create both commands and vectorNetwork blobs
        let mut commands_bytes = Vec::new();
        commands_bytes.push(1); // M
        commands_bytes.extend_from_slice(&1.0f32.to_le_bytes());
        commands_bytes.extend_from_slice(&2.0f32.to_le_bytes());
        commands_bytes.push(0); // Z

        let mut network_bytes = Vec::new();
        network_bytes.extend_from_slice(&1u32.to_le_bytes()); // 1 vertex
        network_bytes.extend_from_slice(&0u32.to_le_bytes()); // 0 segments
        network_bytes.extend_from_slice(&0u32.to_le_bytes()); // 0 regions
        network_bytes.extend_from_slice(&0u32.to_le_bytes()); // vertex styleID
        network_bytes.extend_from_slice(&5.0f32.to_le_bytes()); // vertex x
        network_bytes.extend_from_slice(&5.0f32.to_le_bytes()); // vertex y

        let blobs = vec![
            json!({"bytes": commands_bytes}),
            json!({"bytes": network_bytes}),
        ];

        let mut tree = json!({
            "name": "Shape",
            "commandsBlob": 0,
            "vectorNetworkBlob": 1
        });

        substitute_blobs(&mut tree, &blobs).unwrap();

        // Both blob fields should be replaced
        assert!(tree.get("commandsBlob").is_none());
        assert!(tree.get("vectorNetworkBlob").is_none());
        assert!(tree.get("commands").is_some());
        assert!(tree.get("vectorNetwork").is_some());
    }

    #[test]
    fn test_substitute_unknown_blob_type() {
        let blobs = vec![json!({
            "bytes": vec![1, 2, 3, 4]
        })];

        let mut tree = json!({
            "name": "Node",
            "unknownBlob": 0
        });

        // Should not fail, just leave the field as-is
        substitute_blobs(&mut tree, &blobs).unwrap();

        // Unknown blob type should remain unchanged
        assert!(tree.get("unknownBlob").is_some());
        assert_eq!(tree.get("unknownBlob").unwrap().as_u64(), Some(0));
    }

    #[test]
    fn test_substitute_invalid_blob_index() {
        let blobs = vec![json!({
            "bytes": vec![1, 2, 3]
        })];

        let mut tree = json!({
            "name": "Node",
            "commandsBlob": 999  // Out of range
        });

        // Should not fail, just leave the field as-is
        substitute_blobs(&mut tree, &blobs).unwrap();

        // Out of range index should remain unchanged
        assert!(tree.get("commandsBlob").is_some());
        assert_eq!(tree.get("commandsBlob").unwrap().as_u64(), Some(999));
    }

    #[test]
    fn test_substitute_preserves_other_fields() {
        let mut blob_bytes = vec![0]; // Z

        let blobs = vec![json!({
            "bytes": blob_bytes
        })];

        let mut tree = json!({
            "name": "Node",
            "type": "VECTOR",
            "visible": true,
            "commandsBlob": 0,
            "x": 10,
            "y": 20
        });

        substitute_blobs(&mut tree, &blobs).unwrap();

        // Other fields should be preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("VECTOR"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));

        // commandsBlob should be replaced
        assert!(tree.get("commandsBlob").is_none());
        assert!(tree.get("commands").is_some());
    }
}
