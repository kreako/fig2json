use crate::error::{FigError, Result};
use crate::types::ParsedFile;
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Minimum file size for a valid .fig file (8 bytes header + 4 bytes version)
const MIN_FILE_SIZE: usize = 12;

/// Minimum chunk header size (4 bytes for length)
const CHUNK_HEADER_SIZE: usize = 4;

/// Extract canvas.fig from a ZIP archive
///
/// Some .fig files (especially larger ones) are stored as ZIP archives
/// containing a `canvas.fig` file. This function extracts that file.
///
/// # Arguments
/// * `bytes` - Raw ZIP file bytes
///
/// # Returns
/// * `Ok(Vec<u8>)` - Extracted canvas.fig file contents
/// * `Err(FigError)` - If ZIP extraction fails or canvas.fig not found
///
/// # Examples
/// ```no_run
/// use fig2json::parser::extract_from_zip;
///
/// let zip_bytes = std::fs::read("example.fig").unwrap();
/// let canvas_bytes = extract_from_zip(&zip_bytes).unwrap();
/// ```
pub fn extract_from_zip(bytes: &[u8]) -> Result<Vec<u8>> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    // Look for "canvas.fig" entry
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name == "canvas.fig" {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            return Ok(contents);
        }
    }

    Err(FigError::CanvasNotFoundInZip)
}

/// Extract chunks from a .fig file (version format)
///
/// Parses the version format used by Evan Wallace's approach:
/// ```text
/// [8 bytes] Magic header: "fig-kiwi" or "fig-jam."
/// [4 bytes] Version (uint32, little-endian)
/// [4 bytes] Chunk 0 length (uint32, little-endian)
/// [N bytes] Compressed chunk 0 (schema)
/// [4 bytes] Chunk 1 length (uint32, little-endian)
/// [N bytes] Compressed chunk 1 (data)
/// [...] Additional chunks (images, etc.)
/// ```
///
/// # Arguments
/// * `bytes` - Raw .fig file bytes (after magic header validation)
///
/// # Returns
/// * `Ok(ParsedFile)` - Parsed file with version and chunks
/// * `Err(FigError)` - If parsing fails
///
/// # Examples
/// ```no_run
/// use fig2json::parser::extract_chunks;
///
/// let bytes = std::fs::read("example.canvas.fig").unwrap();
/// let parsed = extract_chunks(&bytes).unwrap();
/// println!("Version: {}", parsed.version);
/// println!("Chunks: {}", parsed.chunks.len());
/// ```
pub fn extract_chunks(bytes: &[u8]) -> Result<ParsedFile> {
    // Validate minimum file size
    if bytes.len() < MIN_FILE_SIZE {
        return Err(FigError::FileTooSmall {
            expected: MIN_FILE_SIZE,
            actual: bytes.len(),
        });
    }

    // Read version at offset 8 (after 8-byte magic header)
    let version = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

    // Extract all chunks starting at offset 12
    let mut chunks = Vec::new();
    let mut offset = 12;

    while offset < bytes.len() {
        // Check if we have enough bytes for chunk header
        if offset + CHUNK_HEADER_SIZE > bytes.len() {
            // No more complete chunks, we're done
            break;
        }

        // Read chunk length (4 bytes, little-endian)
        let chunk_length = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += CHUNK_HEADER_SIZE;

        // Validate we have enough bytes for the chunk data
        if offset + chunk_length > bytes.len() {
            return Err(FigError::IncompleteChunk {
                offset: offset - CHUNK_HEADER_SIZE,
                expected: chunk_length,
                actual: bytes.len() - offset,
            });
        }

        // Extract chunk data
        let chunk_data = bytes[offset..offset + chunk_length].to_vec();
        chunks.push(chunk_data);
        offset += chunk_length;
    }

    // Validate we have at least 2 chunks (schema + data)
    if chunks.len() < 2 {
        return Err(FigError::NotEnoughChunks {
            expected: 2,
            actual: chunks.len(),
        });
    }

    Ok(ParsedFile::new(version, chunks))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_chunks_minimal() {
        // Create a minimal valid .fig file structure
        let mut bytes = Vec::new();

        // Magic header
        bytes.extend_from_slice(b"fig-kiwi");

        // Version (48, little-endian)
        bytes.extend_from_slice(&48u32.to_le_bytes());

        // Chunk 0: length 5
        bytes.extend_from_slice(&5u32.to_le_bytes());
        bytes.extend_from_slice(b"chunk");

        // Chunk 1: length 4
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(b"data");

        let result = extract_chunks(&bytes).unwrap();
        assert_eq!(result.version, 48);
        assert_eq!(result.chunks.len(), 2);
        assert_eq!(result.chunks[0], b"chunk");
        assert_eq!(result.chunks[1], b"data");
    }

    #[test]
    fn test_extract_chunks_multiple() {
        // Test with multiple chunks (schema + data + images)
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"fig-kiwi");
        bytes.extend_from_slice(&101u32.to_le_bytes());

        // Three chunks
        for i in 0..3 {
            let chunk_data = format!("chunk{}", i);
            bytes.extend_from_slice(&(chunk_data.len() as u32).to_le_bytes());
            bytes.extend_from_slice(chunk_data.as_bytes());
        }

        let result = extract_chunks(&bytes).unwrap();
        assert_eq!(result.version, 101);
        assert_eq!(result.chunks.len(), 3);
    }

    #[test]
    fn test_extract_chunks_file_too_small() {
        let bytes = b"fig-kiwi\x00";
        let result = extract_chunks(bytes);
        assert!(result.is_err());
        match result {
            Err(FigError::FileTooSmall { .. }) => (),
            _ => panic!("Expected FileTooSmall error"),
        }
    }

    #[test]
    fn test_extract_chunks_incomplete_chunk() {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"fig-kiwi");
        bytes.extend_from_slice(&48u32.to_le_bytes());

        // Chunk with length 100 but only 5 bytes of data
        bytes.extend_from_slice(&100u32.to_le_bytes());
        bytes.extend_from_slice(b"short");

        let result = extract_chunks(&bytes);
        assert!(result.is_err());
        match result {
            Err(FigError::IncompleteChunk { .. }) => (),
            _ => panic!("Expected IncompleteChunk error"),
        }
    }

    #[test]
    fn test_extract_chunks_not_enough_chunks() {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"fig-kiwi");
        bytes.extend_from_slice(&48u32.to_le_bytes());

        // Only one chunk (need at least 2)
        bytes.extend_from_slice(&5u32.to_le_bytes());
        bytes.extend_from_slice(b"chunk");

        let result = extract_chunks(&bytes);
        assert!(result.is_err());
        match result {
            Err(FigError::NotEnoughChunks { expected, actual }) => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 1);
            }
            _ => panic!("Expected NotEnoughChunks error"),
        }
    }
}
