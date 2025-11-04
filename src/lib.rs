//! # fig2json
//!
//! A library for parsing Figma `.fig` files and converting them to JSON.
//!
//! ## Example
//!
//! ```no_run
//! use fig2json::parser::{is_zip_container, extract_from_zip, detect_file_type, extract_chunks};
//!
//! let bytes = std::fs::read("example.fig").unwrap();
//!
//! // Check if it's a ZIP container
//! let bytes = if is_zip_container(&bytes) {
//!     extract_from_zip(&bytes).unwrap()
//! } else {
//!     bytes
//! };
//!
//! // Detect file type
//! let file_type = detect_file_type(&bytes).unwrap();
//! println!("File type: {:?}", file_type);
//!
//! // Extract chunks
//! let parsed = extract_chunks(&bytes).unwrap();
//! println!("Version: {}", parsed.version);
//! println!("Number of chunks: {}", parsed.chunks.len());
//! ```

pub mod blobs;
pub mod error;
pub mod parser;
pub mod schema;
pub mod types;

// Re-export commonly used items
pub use error::{FigError, Result};
pub use types::{FileType, ParsedFile};

/// Convert a .fig file to JSON
///
/// This is the main entry point for converting Figma .fig files to JSON format.
/// It handles all phases of the conversion:
/// 1. ZIP extraction (if needed)
/// 2. File type detection
/// 3. Chunk extraction
/// 4. Decompression
/// 5. Kiwi schema decoding
/// 6. Tree building from nodeChanges
/// 7. Blob base64 encoding
/// 8. Blob substitution (replace blob indices with parsed content)
/// 9. Image hash transformation (convert hash arrays to filename strings)
/// 10. Matrix to CSS transformation (convert 2D affine matrices to CSS properties)
/// 11. Color to CSS transformation (convert RGBA color objects to CSS hex strings)
/// 12. Text glyphs removal (remove glyph vector data from text objects)
/// 13. Enum simplification (convert verbose enum objects to simple strings)
/// 14. GUID removal (remove internal Figma identifiers)
/// 15. Edit info removal (remove version control metadata)
/// 16. Phase removal (remove Figma internal state)
/// 17. Geometry removal (remove detailed path commands)
/// 18. Text layout removal (remove detailed text layout data)
/// 19. Text metadata removal (remove text configuration metadata)
/// 20. Stroke properties removal (remove CSS-incompatible stroke properties)
/// 21. Frame properties removal (remove frame-specific metadata)
/// 22. Image metadata removal (remove image metadata fields)
/// 23. Document properties removal (remove document-level properties)
/// 24. Root blobs removal (remove now-unnecessary blobs array from output)
///
/// # Arguments
/// * `bytes` - Raw bytes from the .fig file
///
/// # Returns
/// * `Ok(serde_json::Value)` - JSON representation with document tree and metadata
/// * `Err(FigError)` - If conversion fails at any stage
///
/// # Example
/// ```no_run
/// use fig2json::convert;
///
/// let bytes = std::fs::read("example.fig").unwrap();
/// let json = convert(&bytes).unwrap();
/// println!("{}", serde_json::to_string_pretty(&json).unwrap());
/// ```
pub fn convert(bytes: &[u8]) -> Result<serde_json::Value> {
    // 1. Detect and extract from ZIP if needed
    let bytes = if parser::is_zip_container(bytes) {
        parser::extract_from_zip(bytes)?
    } else {
        bytes.to_vec()
    };

    // 2. Detect file type (figma vs figjam)
    let file_type = parser::detect_file_type(&bytes)?;

    // 3. Extract chunks (version format)
    let parsed = parser::extract_chunks(&bytes)?;

    // 4. Decompress chunks
    let schema_bytes = parser::decompress_chunk(parsed.schema_chunk().ok_or({
        FigError::NotEnoughChunks {
            expected: 1,
            actual: 0,
        }
    })?)?;
    let data_bytes = parser::decompress_chunk(parsed.data_chunk().ok_or({
        FigError::NotEnoughChunks {
            expected: 2,
            actual: parsed.chunks.len(),
        }
    })?)?;

    // 5. Decode with Kiwi schema
    let json = schema::decode_fig_to_json(&schema_bytes, &data_bytes)?;

    // 6. Extract nodeChanges and build tree structure
    let node_changes = json
        .get("nodeChanges")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FigError::ZipError("No nodeChanges found in decoded data".to_string()))?
        .clone();

    let mut document = schema::build_tree(node_changes)?;

    // 7. Extract and process blobs (convert to base64)
    let blobs = json
        .get("blobs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FigError::ZipError("No blobs found in decoded data".to_string()))?
        .clone();

    let processed_blobs = blobs::process_blobs(blobs)?;

    // 8. Substitute blob references in document tree with parsed blob content
    // This replaces fields like "commandsBlob: 5" with "commands: [parsed array]"
    blobs::substitute_blobs(&mut document, processed_blobs.as_array().unwrap())?;

    // 9. Transform image hash arrays to filename strings
    // This converts "image.hash: [96, 73, ...]" to "image.filename: images/6049..."
    schema::transform_image_hashes(&mut document)?;

    // 10. Transform 2D affine transformation matrices to CSS properties
    // This converts "transform: {m00, m01, m02, m10, m11, m12}" to "transform: {x, y, rotation, scaleX, scaleY, skewX}"
    schema::transform_matrix_to_css(&mut document)?;

    // 11. Transform RGBA color objects to CSS hex strings
    // This converts "color: {r, g, b, a}" to "color: #rrggbb" or "color: #rrggbbaa"
    schema::transform_colors_to_css(&mut document)?;

    // 12. Remove text glyph vector data
    // This removes "glyphs" arrays from "derivedTextData" objects to reduce output size
    schema::remove_text_glyphs(&mut document)?;

    // 13. Simplify enum objects to simple strings
    // This converts {"__enum__": "NodeType", "value": "FRAME"} to "FRAME"
    schema::simplify_enums(&mut document)?;

    // 14. Remove GUID fields (internal Figma identifiers)
    schema::remove_guid_fields(&mut document)?;

    // 15. Remove editInfo fields (version control metadata)
    schema::remove_edit_info_fields(&mut document)?;

    // 16. Remove phase fields (Figma internal state)
    schema::remove_phase_fields(&mut document)?;

    // 17. Remove geometry fields (detailed path commands)
    schema::remove_geometry_fields(&mut document)?;

    // 18. Remove text layout fields (detailed text layout data)
    schema::remove_text_layout_fields(&mut document)?;

    // 19. Remove text metadata fields (text configuration metadata)
    schema::remove_text_metadata_fields(&mut document)?;

    // 20. Remove stroke properties (CSS-incompatible stroke properties)
    schema::remove_stroke_properties(&mut document)?;

    // 21. Remove frame properties (frame-specific metadata)
    schema::remove_frame_properties(&mut document)?;

    // 22. Remove image metadata fields (image metadata)
    schema::remove_image_metadata_fields(&mut document)?;

    // Build final JSON output
    let mut output = serde_json::json!({
        "version": parsed.version,
        "fileType": match file_type {
            FileType::Figma => "figma",
            FileType::FigJam => "figjam",
        },
        "document": document,
        "blobs": processed_blobs,
    });

    // 23. Remove document properties (document-level properties)
    schema::remove_document_properties(&mut output)?;

    // 24. Remove root-level blobs array (no longer needed after substitution)
    schema::remove_root_blobs(&mut output)?;

    Ok(output)
}
