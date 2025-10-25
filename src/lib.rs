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
///
/// # Arguments
/// * `bytes` - Raw bytes from the .fig file
///
/// # Returns
/// * `Ok(serde_json::Value)` - JSON representation with document tree, blobs, and metadata
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
    let schema_bytes = parser::decompress_chunk(parsed.schema_chunk().ok_or_else(|| {
        FigError::NotEnoughChunks {
            expected: 1,
            actual: 0,
        }
    })?)?;
    let data_bytes = parser::decompress_chunk(parsed.data_chunk().ok_or_else(|| {
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

    let document = schema::build_tree(node_changes)?;

    // 7. Extract and process blobs (convert to base64)
    let blobs = json
        .get("blobs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FigError::ZipError("No blobs found in decoded data".to_string()))?
        .clone();

    let processed_blobs = blobs::process_blobs(blobs)?;

    // Build final JSON output
    Ok(serde_json::json!({
        "version": parsed.version,
        "fileType": match file_type {
            FileType::Figma => "figma",
            FileType::FigJam => "figjam",
        },
        "document": document,
        "blobs": processed_blobs,
    }))
}
