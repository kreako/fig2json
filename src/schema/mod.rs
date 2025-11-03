pub mod decoder;
pub mod transformations;
pub mod tree;

// Re-export commonly used items
pub use decoder::decode_fig_to_json;
pub use transformations::{
    remove_root_blobs, remove_text_glyphs, transform_image_hashes, transform_matrix_to_css,
};
pub use tree::build_tree;
