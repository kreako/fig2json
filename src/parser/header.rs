use crate::error::{FigError, Result};
use crate::types::FileType;

/// Magic header for standard Figma files
const FIGMA_MAGIC: &[u8; 8] = b"fig-kiwi";

/// Magic header for FigJam files
const FIGJAM_MAGIC: &[u8; 8] = b"fig-jam.";

/// ZIP magic signature (first two bytes)
const ZIP_MAGIC: &[u8; 2] = b"PK";

/// Minimum file size for a valid .fig file (8 bytes header + 4 bytes version)
const MIN_FILE_SIZE: usize = 12;

/// Detect the file type based on magic header
///
/// # Arguments
/// * `bytes` - Raw file bytes to analyze
///
/// # Returns
/// * `Ok(FileType)` - Detected file type (Figma or FigJam)
/// * `Err(FigError)` - If file is too small or has invalid header
///
/// # Examples
/// ```
/// use fig2json::parser::detect_file_type;
///
/// let bytes = b"fig-kiwi\x00\x00\x00\x00...";
/// let file_type = detect_file_type(bytes).unwrap();
/// ```
pub fn detect_file_type(bytes: &[u8]) -> Result<FileType> {
    if bytes.len() < 8 {
        return Err(FigError::FileTooSmall {
            expected: 8,
            actual: bytes.len(),
        });
    }

    let header = &bytes[0..8];

    // Check for "fig-kiwi" (standard Figma)
    if header == FIGMA_MAGIC {
        return Ok(FileType::Figma);
    }

    // Check for "fig-jam." (FigJam)
    if header == FIGJAM_MAGIC {
        return Ok(FileType::FigJam);
    }

    // Invalid header
    Err(FigError::InvalidMagicHeader(header.to_vec()))
}

/// Check if the file is a ZIP container
///
/// Some .fig files are ZIP archives containing a `canvas.fig` file inside.
/// This function checks for the ZIP magic signature "PK" (0x50 0x4B).
///
/// # Arguments
/// * `bytes` - Raw file bytes to analyze
///
/// # Returns
/// * `true` - If file starts with ZIP signature
/// * `false` - Otherwise
///
/// # Examples
/// ```
/// use fig2json::parser::is_zip_container;
///
/// let zip_bytes = b"PK\x03\x04...";
/// assert!(is_zip_container(zip_bytes));
///
/// let fig_bytes = b"fig-kiwi...";
/// assert!(!is_zip_container(fig_bytes));
/// ```
pub fn is_zip_container(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && &bytes[0..2] == ZIP_MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_figma_header() {
        let bytes = b"fig-kiwi\x00\x00\x00\x00";
        let result = detect_file_type(bytes).unwrap();
        assert_eq!(result, FileType::Figma);
    }

    #[test]
    fn test_detect_figjam_header() {
        let bytes = b"fig-jam.\x00\x00\x00\x00";
        let result = detect_file_type(bytes).unwrap();
        assert_eq!(result, FileType::FigJam);
    }

    #[test]
    fn test_invalid_header() {
        let bytes = b"invalid!";
        let result = detect_file_type(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_too_small() {
        let bytes = b"fig";
        let result = detect_file_type(bytes);
        assert!(result.is_err());
        match result {
            Err(FigError::FileTooSmall { expected, actual }) => {
                assert_eq!(expected, 8);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected FileTooSmall error"),
        }
    }

    #[test]
    fn test_is_zip_container() {
        // Valid ZIP signature
        let zip_bytes = b"PK\x03\x04";
        assert!(is_zip_container(zip_bytes));

        // Not a ZIP
        let fig_bytes = b"fig-kiwi";
        assert!(!is_zip_container(fig_bytes));

        // Too small
        let small_bytes = b"P";
        assert!(!is_zip_container(small_bytes));
    }
}
