/// Transformation passes for the final JSON output
///
/// This module contains various transformation passes that are applied to the
/// JSON document after initial parsing and blob substitution:
///
/// - `image_hash`: Convert image hash arrays to filename strings
/// - `blobs_removal`: Remove the root-level blobs array from final output
/// - `matrix_to_css`: Convert 2D affine transformation matrices to CSS positioning properties
/// - `color_to_css`: Convert RGBA color objects to CSS hex color strings
/// - `text_glyphs_removal`: Remove glyph vector data from text objects
/// - `guid_removal`: Remove internal Figma guid identifiers
/// - `edit_info_removal`: Remove version control edit info metadata
/// - `phase_removal`: Remove Figma internal phase state
/// - `geometry_removal`: Remove detailed geometry path commands
/// - `text_layout_removal`: Remove detailed text layout data
/// - `text_metadata_removal`: Remove text configuration metadata
/// - `stroke_properties_removal`: Remove CSS-incompatible stroke properties
/// - `frame_properties_removal`: Remove frame-specific metadata
/// - `image_metadata_removal`: Remove image metadata fields
/// - `document_properties_removal`: Remove document-level properties
/// - `enum_simplification`: Simplify verbose enum objects to simple strings
/// - `text_data_removal`: Remove Figma-specific textData field
/// - `default_text_properties_removal`: Remove default text property values
/// - `empty_font_postscript_removal`: Remove empty postscript from fontName
/// - `border_weights_removal`: Remove individual border weight fields
/// - `default_blend_mode_removal`: Remove default blendMode values
/// - `background_properties_removal`: Remove background metadata fields
/// - `internal_only_nodes_removal`: Filter out internal-only nodes
/// - `derived_text_layout_size_removal`: Remove redundant layoutSize from derivedTextData
pub mod background_properties_removal;
pub mod blobs_removal;
pub mod border_weights_removal;
pub mod color_to_css;
pub mod default_blend_mode_removal;
pub mod default_text_properties_removal;
pub mod derived_text_layout_size_removal;
pub mod document_properties_removal;
pub mod edit_info_removal;
pub mod empty_font_postscript_removal;
pub mod enum_simplification;
pub mod frame_properties_removal;
pub mod geometry_removal;
pub mod guid_removal;
pub mod image_hash;
pub mod image_metadata_removal;
pub mod internal_only_nodes_removal;
pub mod matrix_to_css;
pub mod phase_removal;
pub mod stroke_properties_removal;
pub mod text_data_removal;
pub mod text_glyphs_removal;
pub mod text_layout_removal;
pub mod text_metadata_removal;

// Re-export commonly used functions
pub use background_properties_removal::remove_background_properties;
pub use blobs_removal::remove_root_blobs;
pub use border_weights_removal::remove_border_weights;
pub use color_to_css::transform_colors_to_css;
pub use default_blend_mode_removal::remove_default_blend_mode;
pub use default_text_properties_removal::remove_default_text_properties;
pub use derived_text_layout_size_removal::remove_derived_text_layout_size;
pub use document_properties_removal::remove_document_properties;
pub use edit_info_removal::remove_edit_info_fields;
pub use empty_font_postscript_removal::remove_empty_font_postscript;
pub use enum_simplification::simplify_enums;
pub use frame_properties_removal::remove_frame_properties;
pub use geometry_removal::remove_geometry_fields;
pub use guid_removal::remove_guid_fields;
pub use image_hash::transform_image_hashes;
pub use image_metadata_removal::remove_image_metadata_fields;
pub use internal_only_nodes_removal::remove_internal_only_nodes;
pub use matrix_to_css::transform_matrix_to_css;
pub use phase_removal::remove_phase_fields;
pub use stroke_properties_removal::remove_stroke_properties;
pub use text_data_removal::remove_text_data_fields;
pub use text_glyphs_removal::remove_text_glyphs;
pub use text_layout_removal::remove_text_layout_fields;
pub use text_metadata_removal::remove_text_metadata_fields;
