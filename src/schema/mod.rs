pub mod decoder;
pub mod transformations;
pub mod tree;

// Re-export commonly used items
pub use decoder::decode_fig_to_json;
pub use transformations::{remove_root_blobs, transform_image_hashes};
pub use tree::build_tree;
