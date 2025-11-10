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
/// 20. Default text line properties removal (remove default values from textData.lines arrays)
/// 21. Default text properties removal (remove default letterSpacing/lineHeight values)
/// 22. Text properties simplification (convert verbose letterSpacing/lineHeight to CSS strings)
/// 23. Empty font postscript removal (remove empty postscript from fontName)
/// 24. Stroke properties removal (remove CSS-incompatible stroke properties)
/// 25. Border weights removal (remove individual border weight fields)
/// 26. Frame properties removal (remove frame-specific metadata)
/// 27. Background properties removal (remove backgroundEnabled, backgroundOpacity)
/// 28. Image metadata removal (remove image metadata fields)
/// 29. Internal-only nodes removal (filter out internalOnly: true nodes)
/// 30. Default opacity removal (remove opacity: 1.0)
/// 31. Default visible removal (remove visible: true)
/// 32. Default rotation removal (remove rotation: 0.0)
/// 33. Document properties removal (remove document-level properties)
/// 34. Root metadata removal (remove version and fileType fields)
/// 35. Root blobs removal (remove now-unnecessary blobs array from output)
/// 36. GUID path removal (remove internal Figma guidPath references)
/// 37. User facing version removal (remove Figma version strings)
/// 38. Style ID removal (remove Figma shared style references)
/// 39. Export settings removal (remove asset export configurations)
/// 40. Plugin data removal (remove Figma plugin storage data)
/// 41. Rectangle corner radii independent removal (remove corner radii independent flag)
/// 42. Constraint properties removal (remove Figma auto-layout constraint properties)
/// 43. Scroll/resize properties removal (remove Figma scroll and resize behavior properties)
/// 44. Layout aids removal (remove design-time layout aids like guides and layoutGrids)
/// 45. Detached symbol ID removal (remove Figma component instance metadata)
/// 46. Overridden symbol ID removal (remove standalone overriddenSymbolID objects from arrays)
/// 47. Redundant corner radii removal (remove individual corner radius fields when general cornerRadius exists)
/// 48. Corner smoothing removal (remove Figma's corner smoothing property)
/// 49. Invisible paints removal (remove invisible paints from fillPaints and strokePaints arrays)
/// 50. Empty paint arrays removal (remove empty fillPaints and strokePaints arrays)
/// 51. Redundant padding removal (remove stackPaddingRight/stackPaddingBottom when axis-based padding exists)
/// 52. Stack child properties removal (remove stackChildAlignSelf and stackChildPrimaryGrow)
/// 53. Stack sizing properties removal (remove stackCounterSizing and stackPrimarySizing)
/// 54. Stack alignment properties removal (remove stackCounterAlignItems and stackPrimaryAlignItems)
/// 55. Type removal (remove type field from all nodes)
/// 56. Empty objects removal (remove empty objects {} from the JSON tree)
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

    // 23. Remove default text line properties (default values from textData.lines arrays)
    schema::remove_default_text_line_properties(&mut document)?;

    // 24. Remove default text properties (letterSpacing 0%, lineHeight 100%)
    schema::remove_default_text_properties(&mut document)?;

    // 25. Simplify text properties (convert verbose letterSpacing/lineHeight to CSS strings)
    schema::simplify_text_properties(&mut document)?;

    // 26. Remove empty postscript from fontName objects
    schema::remove_empty_font_postscript(&mut document)?;

    // 27. Remove stroke properties (CSS-incompatible stroke properties)
    schema::remove_stroke_properties(&mut document)?;

    // 28. Remove border weight fields (CSS-incompatible individual border weights)
    schema::remove_border_weights(&mut document)?;

    // 29. Remove frame properties (frame-specific metadata)
    schema::remove_frame_properties(&mut document)?;

    // 30. Remove background properties (backgroundEnabled, backgroundOpacity)
    schema::remove_background_properties(&mut document)?;

    // 31. Remove image metadata fields (image metadata, including imageThumbnail)
    schema::remove_image_metadata_fields(&mut document)?;

    // 32. Remove internal-only nodes (filter out internalOnly: true nodes)
    schema::remove_internal_only_nodes(&mut document)?;

    // 33. Remove default opacity values (1.0 is the default)
    schema::remove_default_opacity(&mut document)?;

    // 34. Remove default visible values (true is the default)
    schema::remove_default_visible(&mut document)?;

    // 35. Remove default rotation values (0.0 is the default)
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

    // 36. Remove document properties (document-level properties)
    schema::remove_document_properties(&mut output)?;

    // 37. Remove root-level metadata fields (version and fileType)
    schema::remove_root_metadata(&mut output)?;

    // 38. Remove root-level blobs array (no longer needed after substitution)
    schema::remove_root_blobs(&mut output)?;

    // 39. Remove guid paths (internal Figma guidPath references)
    schema::remove_guid_paths(&mut output)?;

    // 40. Remove user facing versions (Figma version strings)
    schema::remove_user_facing_versions(&mut output)?;

    // 41. Remove style IDs (Figma shared style references)
    schema::remove_style_ids(&mut output)?;

    // 42. Remove export settings (asset export configurations)
    schema::remove_export_settings(&mut output)?;

    // 43. Remove plugin data (Figma plugin storage data)
    schema::remove_plugin_data(&mut output)?;

    // 44. Remove rectangle corner radii independent (corner radii independent flag)
    schema::remove_rectangle_corner_radii_independent(&mut output)?;

    // 45. Remove constraint properties (horizontalConstraint, verticalConstraint)
    schema::remove_constraint_properties(&mut output)?;

    // 46. Remove scroll/resize properties (scrollBehavior, resizeToFit)
    schema::remove_scroll_resize_properties(&mut output)?;

    // 47. Remove layout aids (guides, layoutGrids)
    schema::remove_layout_aids(&mut output)?;

    // 48. Remove detached symbol ID (Figma component instance metadata)
    schema::remove_detached_symbol_id(&mut output)?;

    // 49. Remove standalone overriddenSymbolID objects (Figma component swap metadata)
    schema::remove_overridden_symbol_id(&mut output)?;

    // 50. Remove redundant corner radii (individual corner radius fields when cornerRadius exists)
    schema::remove_redundant_corner_radii(&mut output)?;

    // 51. Remove corner smoothing (Figma's corner smoothing property)
    schema::remove_corner_smoothing(&mut output)?;

    // 52. Remove invisible paints (filter out paints with visible: false)
    schema::remove_invisible_paints(&mut output)?;

    // 53. Remove empty paint arrays (remove empty fillPaints and strokePaints arrays)
    schema::remove_empty_paint_arrays(&mut output)?;

    // 54. Remove redundant padding properties (stackPaddingRight/stackPaddingBottom when axis-based padding exists)
    schema::remove_redundant_padding(&mut output)?;

    // 55. Remove stack child properties (stackChildAlignSelf, stackChildPrimaryGrow)
    schema::remove_stack_child_properties(&mut output)?;

    // 56. Remove stack sizing properties (stackCounterSizing, stackPrimarySizing)
    schema::remove_stack_sizing_properties(&mut output)?;

    // 57. Remove stack alignment properties (stackCounterAlignItems, stackPrimaryAlignItems)
    schema::remove_stack_align_items(&mut output)?;

    // 58. Remove type field from all nodes
    schema::remove_type(&mut output)?;

    // 59. Remove empty objects {} from the JSON tree
    schema::remove_empty_objects(&mut output)?;

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
