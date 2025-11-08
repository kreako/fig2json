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
/// 24. Empty derivedTextData removal (remove empty derivedTextData objects)
/// 25. Default opacity removal (remove opacity: 1.0)
/// 26. Default visible removal (remove visible: true)
/// 27. Default rotation removal (remove rotation: 0.0)
/// 28. Root metadata removal (remove version and fileType fields)
/// 29. Root blobs removal (remove now-unnecessary blobs array from output)
/// 30. Symbol data removal (remove Figma component instance metadata)
/// 31. Derived symbol data removal (remove derived symbol data and layout version)
/// 32. GUID path removal (remove internal Figma guidPath references)
/// 33. User facing version removal (remove Figma version strings)
/// 34. Style ID removal (remove Figma shared style references)
/// 35. Export settings removal (remove asset export configurations)
/// 36. Plugin data removal (remove Figma plugin storage data)
/// 37. Component properties removal (remove component property assignments)
/// 38. Rectangle corner radii independent removal (remove corner radii independent flag)
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

    // 14. Remove default blendMode when "NORMAL" (must run after enum simplification)
    // This removes blendMode fields with default "NORMAL" value to reduce output size
    schema::remove_default_blend_mode(&mut document)?;

    // 15. Remove GUID fields (internal Figma identifiers)
    schema::remove_guid_fields(&mut document)?;

    // 16. Remove editInfo fields (version control metadata)
    schema::remove_edit_info_fields(&mut document)?;

    // 17. Remove phase fields (Figma internal state)
    schema::remove_phase_fields(&mut document)?;

    // 18. Remove geometry fields (detailed path commands)
    schema::remove_geometry_fields(&mut document)?;

    // 19. Remove text layout fields (detailed text layout data)
    schema::remove_text_layout_fields(&mut document)?;

    // 20. Remove layoutSize from derivedTextData (redundant with node size)
    schema::remove_derived_text_layout_size(&mut document)?;

    // 21. Remove empty derivedTextData objects (no useful information for HTML/CSS)
    schema::remove_empty_derived_text_data(&mut document)?;

    // 22. Remove text metadata fields (text configuration metadata)
    schema::remove_text_metadata_fields(&mut document)?;

    // 23. Remove textData field (Figma-specific line metadata)
    schema::remove_text_data_fields(&mut document)?;

    // 24. Remove default text properties (letterSpacing 0%, lineHeight 100%)
    schema::remove_default_text_properties(&mut document)?;

    // 25. Remove empty postscript from fontName objects
    schema::remove_empty_font_postscript(&mut document)?;

    // 26. Remove stroke properties (CSS-incompatible stroke properties)
    schema::remove_stroke_properties(&mut document)?;

    // 27. Remove border weight fields (CSS-incompatible individual border weights)
    schema::remove_border_weights(&mut document)?;

    // 28. Remove frame properties (frame-specific metadata)
    schema::remove_frame_properties(&mut document)?;

    // 29. Remove background properties (backgroundEnabled, backgroundOpacity)
    schema::remove_background_properties(&mut document)?;

    // 30. Remove image metadata fields (image metadata, including imageThumbnail)
    schema::remove_image_metadata_fields(&mut document)?;

    // 31. Remove internal-only nodes (filter out internalOnly: true nodes)
    schema::remove_internal_only_nodes(&mut document)?;

    // 32. Remove default opacity values (1.0 is the default)
    schema::remove_default_opacity(&mut document)?;

    // 33. Remove default visible values (true is the default)
    schema::remove_default_visible(&mut document)?;

    // 34. Remove default rotation values (0.0 is the default)
    schema::remove_default_rotation(&mut document)?;

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

    // 35. Remove document properties (document-level properties)
    schema::remove_document_properties(&mut output)?;

    // 36. Remove root-level metadata fields (version and fileType)
    schema::remove_root_metadata(&mut output)?;

    // 37. Remove root-level blobs array (no longer needed after substitution)
    schema::remove_root_blobs(&mut output)?;

    // 38. Remove symbol data (Figma component instance metadata)
    schema::remove_symbol_data(&mut output)?;

    // 39. Remove derived symbol data (derived symbol data and layout version)
    schema::remove_derived_symbol_data(&mut output)?;

    // 40. Remove guid paths (internal Figma guidPath references)
    schema::remove_guid_paths(&mut output)?;

    // 41. Remove user facing versions (Figma version strings)
    schema::remove_user_facing_versions(&mut output)?;

    // 42. Remove style IDs (Figma shared style references)
    schema::remove_style_ids(&mut output)?;

    // 43. Remove export settings (asset export configurations)
    schema::remove_export_settings(&mut output)?;

    // 44. Remove plugin data (Figma plugin storage data)
    schema::remove_plugin_data(&mut output)?;

    // 45. Remove component properties (component property assignments)
    schema::remove_component_properties(&mut output)?;

    // 46. Remove rectangle corner radii independent (corner radii independent flag)
    schema::remove_rectangle_corner_radii_independent(&mut output)?;

    Ok(output)
}

/// Convert a .fig file to raw JSON without transformations
///
/// This function is similar to `convert()` but stops before applying any transformations.
/// It provides the raw Figma data structure without optimization for HTML/CSS conversion.
///
/// The raw output includes all Figma-specific fields and internal data structures that
/// are typically removed or simplified in the standard conversion process.
///
/// # Arguments
/// * `bytes` - Raw bytes from the .fig file
///
/// # Returns
/// * `Ok(serde_json::Value)` - Raw JSON representation with full Figma data
/// * `Err(FigError)` - If conversion fails at any stage
///
/// # Example
/// ```no_run
/// use fig2json::convert_raw;
///
/// let bytes = std::fs::read("example.fig").unwrap();
/// let json = convert_raw(&bytes).unwrap();
/// println!("{}", serde_json::to_string_pretty(&json).unwrap());
/// ```
pub fn convert_raw(bytes: &[u8]) -> Result<serde_json::Value> {
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

    // Build final JSON output WITHOUT transformations
    let output = serde_json::json!({
        "version": parsed.version,
        "fileType": match file_type {
            FileType::Figma => "figma",
            FileType::FigJam => "figjam",
        },
        "document": document,
        "blobs": processed_blobs,
    });

    Ok(output)
}
