use crate::error::Result;
use serde_json::Value as JsonValue;

/// Parse a blob based on its type
///
/// Takes a blob type (field name without "Blob" suffix) and the blob object,
/// extracts the bytes, and parses them into structured JSON data.
///
/// # Arguments
/// * `blob_type` - The type of blob (e.g., "commands", "vectorNetwork")
/// * `blob` - The blob object containing bytes (may be base64 string or byte array)
///
/// # Returns
/// * `Ok(Some(JsonValue))` - Successfully parsed blob data
/// * `Ok(None)` - Unknown blob type or unparseable data
/// * `Err(FigError)` - If blob extraction fails
pub fn parse_blob(blob_type: &str, blob: &JsonValue) -> Result<Option<JsonValue>> {
    // Extract bytes from blob object
    let bytes = extract_blob_bytes(blob)?;

    // Parse based on type
    match blob_type {
        "commands" => Ok(parse_commands(&bytes)),
        "vectorNetwork" => Ok(parse_vector_network(&bytes)),
        _ => Ok(None), // Unknown blob type, return None
    }
}

/// Extract bytes from a blob object
///
/// Blobs can be stored as:
/// - Base64 string in "bytes" field
/// - Array of numbers in "bytes" field
fn extract_blob_bytes(blob: &JsonValue) -> Result<Vec<u8>> {
    let bytes_value = blob
        .get("bytes")
        .ok_or_else(|| crate::error::FigError::ZipError("Blob missing bytes field".to_string()))?;

    // Handle base64 string
    if let Some(base64_str) = bytes_value.as_str() {
        use base64::{engine::general_purpose, Engine as _};
        return general_purpose::STANDARD
            .decode(base64_str)
            .map_err(|e| crate::error::FigError::ZipError(format!("Failed to decode base64: {}", e)));
    }

    // Handle array of numbers
    if let Some(bytes_array) = bytes_value.as_array() {
        let bytes: Vec<u8> = bytes_array
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();
        return Ok(bytes);
    }

    Err(crate::error::FigError::ZipError(
        "Blob bytes field is neither string nor array".to_string(),
    ))
}

/// Parse binary path commands into JSON array
///
/// Converts binary path command data into a flat JSON array in the format:
/// `["M", x, y, "L", x, y, "Q", cx, cy, x, y, "C", cx1, cy1, cx2, cy2, x, y, "Z"]`
///
/// Command types:
/// - 0: Z (close path, no coordinates)
/// - 1: M (move to, 2 floats: x, y)
/// - 2: L (line to, 2 floats: x, y)
/// - 3: Q (quadratic curve, 4 floats: cx, cy, x, y)
/// - 4: C (cubic curve, 6 floats: cx1, cy1, cx2, cy2, x, y)
///
/// All coordinates are stored as little-endian f32 values.
///
/// # Arguments
/// * `bytes` - Binary command data
///
/// # Returns
/// * `Some(JsonValue)` - Array of commands and coordinates
/// * `None` - If data is invalid or incomplete
pub fn parse_commands(bytes: &[u8]) -> Option<JsonValue> {
    let mut commands = Vec::new();
    let mut offset = 0;

    while offset < bytes.len() {
        let cmd_type = bytes[offset];
        offset += 1;

        match cmd_type {
            0 => {
                // Z - close path
                commands.push(JsonValue::String("Z".to_string()));
            }
            1 => {
                // M - move to (x, y)
                if offset + 8 > bytes.len() {
                    return None;
                }
                let x = f32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                let y = f32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                offset += 8;
                commands.push(JsonValue::String("M".to_string()));
                commands.push(json_number(x));
                commands.push(json_number(y));
            }
            2 => {
                // L - line to (x, y)
                if offset + 8 > bytes.len() {
                    return None;
                }
                let x = f32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                let y = f32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                offset += 8;
                commands.push(JsonValue::String("L".to_string()));
                commands.push(json_number(x));
                commands.push(json_number(y));
            }
            3 => {
                // Q - quadratic curve (cx, cy, x, y)
                if offset + 16 > bytes.len() {
                    return None;
                }
                let cx = f32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                let cy = f32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                let x = f32::from_le_bytes([
                    bytes[offset + 8],
                    bytes[offset + 9],
                    bytes[offset + 10],
                    bytes[offset + 11],
                ]);
                let y = f32::from_le_bytes([
                    bytes[offset + 12],
                    bytes[offset + 13],
                    bytes[offset + 14],
                    bytes[offset + 15],
                ]);
                offset += 16;
                commands.push(JsonValue::String("Q".to_string()));
                commands.push(json_number(cx));
                commands.push(json_number(cy));
                commands.push(json_number(x));
                commands.push(json_number(y));
            }
            4 => {
                // C - cubic curve (cx1, cy1, cx2, cy2, x, y)
                if offset + 24 > bytes.len() {
                    return None;
                }
                let cx1 = f32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                let cy1 = f32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                let cx2 = f32::from_le_bytes([
                    bytes[offset + 8],
                    bytes[offset + 9],
                    bytes[offset + 10],
                    bytes[offset + 11],
                ]);
                let cy2 = f32::from_le_bytes([
                    bytes[offset + 12],
                    bytes[offset + 13],
                    bytes[offset + 14],
                    bytes[offset + 15],
                ]);
                let x = f32::from_le_bytes([
                    bytes[offset + 16],
                    bytes[offset + 17],
                    bytes[offset + 18],
                    bytes[offset + 19],
                ]);
                let y = f32::from_le_bytes([
                    bytes[offset + 20],
                    bytes[offset + 21],
                    bytes[offset + 22],
                    bytes[offset + 23],
                ]);
                offset += 24;
                commands.push(JsonValue::String("C".to_string()));
                commands.push(json_number(cx1));
                commands.push(json_number(cy1));
                commands.push(json_number(cx2));
                commands.push(json_number(cy2));
                commands.push(json_number(x));
                commands.push(json_number(y));
            }
            _ => {
                // Unknown command type
                return None;
            }
        }
    }

    Some(JsonValue::Array(commands))
}

/// Parse binary vector network into JSON object
///
/// Converts binary vector network data into a structured JSON object:
/// ```json
/// {
///   "vertices": [{"styleID": 0, "x": 1.0, "y": 2.0}, ...],
///   "segments": [{"styleID": 0, "start": {...}, "end": {...}}, ...],
///   "regions": [{"styleID": 0, "windingRule": "ODD", "loops": [...]}, ...]
/// }
/// ```
///
/// Binary format:
/// - Header: vertexCount (u32), segmentCount (u32), regionCount (u32)
/// - Vertices: styleID (u32), x (f32), y (f32) - repeated vertexCount times
/// - Segments: styleID (u32), startVertex (u32), start.dx (f32), start.dy (f32),
///   endVertex (u32), end.dx (f32), end.dy (f32) - repeated segmentCount times
/// - Regions: styleID+windingRule (u32), loopCount (u32),
///   then for each loop: indexCount (u32), indices (u32[]) - repeated regionCount times
///
/// All values are little-endian.
///
/// # Arguments
/// * `bytes` - Binary vector network data
///
/// # Returns
/// * `Some(JsonValue)` - Object with vertices, segments, and regions
/// * `None` - If data is invalid or incomplete
pub fn parse_vector_network(bytes: &[u8]) -> Option<JsonValue> {
    if bytes.len() < 12 {
        return None;
    }

    // Read header
    let vertex_count = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let segment_count = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let region_count = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;

    let mut offset = 12;

    // Parse vertices
    let mut vertices = Vec::new();
    for _ in 0..vertex_count {
        if offset + 12 > bytes.len() {
            return None;
        }
        let style_id = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let x = f32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        let y = f32::from_le_bytes([
            bytes[offset + 8],
            bytes[offset + 9],
            bytes[offset + 10],
            bytes[offset + 11],
        ]);
        offset += 12;

        vertices.push(serde_json::json!({
            "styleID": style_id,
            "x": json_number(x),
            "y": json_number(y)
        }));
    }

    // Parse segments
    let mut segments = Vec::new();
    for _ in 0..segment_count {
        if offset + 28 > bytes.len() {
            return None;
        }
        let style_id = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let start_vertex = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        let start_dx = f32::from_le_bytes([
            bytes[offset + 8],
            bytes[offset + 9],
            bytes[offset + 10],
            bytes[offset + 11],
        ]);
        let start_dy = f32::from_le_bytes([
            bytes[offset + 12],
            bytes[offset + 13],
            bytes[offset + 14],
            bytes[offset + 15],
        ]);
        let end_vertex = u32::from_le_bytes([
            bytes[offset + 16],
            bytes[offset + 17],
            bytes[offset + 18],
            bytes[offset + 19],
        ]);
        let end_dx = f32::from_le_bytes([
            bytes[offset + 20],
            bytes[offset + 21],
            bytes[offset + 22],
            bytes[offset + 23],
        ]);
        let end_dy = f32::from_le_bytes([
            bytes[offset + 24],
            bytes[offset + 25],
            bytes[offset + 26],
            bytes[offset + 27],
        ]);
        offset += 28;

        // Validate vertex indices
        if start_vertex as usize >= vertex_count || end_vertex as usize >= vertex_count {
            return None;
        }

        segments.push(serde_json::json!({
            "styleID": style_id,
            "start": {
                "vertex": start_vertex,
                "dx": json_number(start_dx),
                "dy": json_number(start_dy)
            },
            "end": {
                "vertex": end_vertex,
                "dx": json_number(end_dx),
                "dy": json_number(end_dy)
            }
        }));
    }

    // Parse regions
    let mut regions = Vec::new();
    for _ in 0..region_count {
        if offset + 8 > bytes.len() {
            return None;
        }

        // styleID and winding rule are packed into one u32
        let style_and_rule = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let winding_rule = if style_and_rule & 1 != 0 {
            "NONZERO"
        } else {
            "ODD"
        };
        let style_id = style_and_rule >> 1;

        let loop_count = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        offset += 8;

        let mut loops = Vec::new();
        for _ in 0..loop_count {
            if offset + 4 > bytes.len() {
                return None;
            }

            let index_count = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + index_count * 4 > bytes.len() {
                return None;
            }

            let mut indices = Vec::new();
            for _ in 0..index_count {
                let segment_index = u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);
                offset += 4;

                // Validate segment index
                if segment_index as usize >= segment_count {
                    return None;
                }

                indices.push(JsonValue::Number(segment_index.into()));
            }

            loops.push(serde_json::json!({
                "segments": indices
            }));
        }

        regions.push(serde_json::json!({
            "styleID": style_id,
            "windingRule": winding_rule,
            "loops": loops
        }));
    }

    Some(serde_json::json!({
        "vertices": vertices,
        "segments": segments,
        "regions": regions
    }))
}

/// Convert f32 to JSON number, handling special values
fn json_number(value: f32) -> JsonValue {
    if value.is_nan() || value.is_infinite() {
        JsonValue::Null
    } else {
        serde_json::Number::from_f64(value as f64)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commands_simple_path() {
        // M 10 20 L 30 40 Z
        let mut bytes = Vec::new();
        bytes.push(1); // M
        bytes.extend_from_slice(&10.0f32.to_le_bytes());
        bytes.extend_from_slice(&20.0f32.to_le_bytes());
        bytes.push(2); // L
        bytes.extend_from_slice(&30.0f32.to_le_bytes());
        bytes.extend_from_slice(&40.0f32.to_le_bytes());
        bytes.push(0); // Z

        let result = parse_commands(&bytes).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 7);
        assert_eq!(arr[0].as_str(), Some("M"));
        assert_eq!(arr[1].as_f64(), Some(10.0));
        assert_eq!(arr[2].as_f64(), Some(20.0));
        assert_eq!(arr[3].as_str(), Some("L"));
        assert_eq!(arr[4].as_f64(), Some(30.0));
        assert_eq!(arr[5].as_f64(), Some(40.0));
        assert_eq!(arr[6].as_str(), Some("Z"));
    }

    #[test]
    fn test_parse_commands_quadratic() {
        // Q 1 2 3 4
        let mut bytes = Vec::new();
        bytes.push(3); // Q
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        bytes.extend_from_slice(&2.0f32.to_le_bytes());
        bytes.extend_from_slice(&3.0f32.to_le_bytes());
        bytes.extend_from_slice(&4.0f32.to_le_bytes());

        let result = parse_commands(&bytes).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_str(), Some("Q"));
        assert_eq!(arr[1].as_f64(), Some(1.0));
        assert_eq!(arr[2].as_f64(), Some(2.0));
        assert_eq!(arr[3].as_f64(), Some(3.0));
        assert_eq!(arr[4].as_f64(), Some(4.0));
    }

    #[test]
    fn test_parse_commands_cubic() {
        // C 1 2 3 4 5 6
        let mut bytes = Vec::new();
        bytes.push(4); // C
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        bytes.extend_from_slice(&2.0f32.to_le_bytes());
        bytes.extend_from_slice(&3.0f32.to_le_bytes());
        bytes.extend_from_slice(&4.0f32.to_le_bytes());
        bytes.extend_from_slice(&5.0f32.to_le_bytes());
        bytes.extend_from_slice(&6.0f32.to_le_bytes());

        let result = parse_commands(&bytes).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 7);
        assert_eq!(arr[0].as_str(), Some("C"));
        assert_eq!(arr[1].as_f64(), Some(1.0));
        assert_eq!(arr[2].as_f64(), Some(2.0));
        assert_eq!(arr[3].as_f64(), Some(3.0));
        assert_eq!(arr[4].as_f64(), Some(4.0));
        assert_eq!(arr[5].as_f64(), Some(5.0));
        assert_eq!(arr[6].as_f64(), Some(6.0));
    }

    #[test]
    fn test_parse_commands_invalid() {
        // Invalid command type
        let bytes = vec![99];
        assert!(parse_commands(&bytes).is_none());

        // Incomplete data
        let bytes = vec![1, 0]; // M with incomplete coordinates
        assert!(parse_commands(&bytes).is_none());
    }

    #[test]
    fn test_parse_vector_network_simple() {
        let mut bytes = Vec::new();

        // Header: 2 vertices, 1 segment, 0 regions
        bytes.extend_from_slice(&2u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());

        // Vertex 0: styleID=0, x=10, y=20
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&10.0f32.to_le_bytes());
        bytes.extend_from_slice(&20.0f32.to_le_bytes());

        // Vertex 1: styleID=0, x=30, y=40
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&30.0f32.to_le_bytes());
        bytes.extend_from_slice(&40.0f32.to_le_bytes());

        // Segment 0: styleID=0, start=(vertex=0, dx=0, dy=0), end=(vertex=1, dx=0, dy=0)
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());

        let result = parse_vector_network(&bytes).unwrap();

        assert!(result.get("vertices").is_some());
        assert!(result.get("segments").is_some());
        assert!(result.get("regions").is_some());

        let vertices = result.get("vertices").unwrap().as_array().unwrap();
        assert_eq!(vertices.len(), 2);

        let segments = result.get("segments").unwrap().as_array().unwrap();
        assert_eq!(segments.len(), 1);

        let regions = result.get("regions").unwrap().as_array().unwrap();
        assert_eq!(regions.len(), 0);
    }
}
