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

pub mod error;
pub mod parser;
pub mod types;

// Re-export commonly used items
pub use error::{FigError, Result};
pub use types::{FileType, ParsedFile};
