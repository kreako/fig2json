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
pub mod blobs_removal;
pub mod color_to_css;
pub mod image_hash;
pub mod matrix_to_css;
pub mod text_glyphs_removal;

// Re-export commonly used functions
pub use blobs_removal::remove_root_blobs;
pub use color_to_css::transform_colors_to_css;
pub use image_hash::transform_image_hashes;
pub use matrix_to_css::transform_matrix_to_css;
pub use text_glyphs_removal::remove_text_glyphs;
