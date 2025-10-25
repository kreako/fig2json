pub mod decoder;
pub mod tree;

// Re-export commonly used items
pub use decoder::decode_fig_to_json;
pub use tree::build_tree;
