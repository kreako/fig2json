/// Type of Figma file based on magic header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Standard Figma file ("fig-kiwi")
    Figma,
    /// FigJam file ("fig-jam.")
    FigJam,
}

/// Parsed .fig file structure with version and chunks
#[derive(Debug, Clone)]
pub struct ParsedFile {
    /// File format version (uint32, little-endian)
    pub version: u32,
    /// Extracted chunks (first chunk is schema, second is data, rest are typically images)
    pub chunks: Vec<Vec<u8>>,
}

impl ParsedFile {
    /// Create a new ParsedFile
    pub fn new(version: u32, chunks: Vec<Vec<u8>>) -> Self {
        Self { version, chunks }
    }

    /// Get the schema chunk (first chunk)
    pub fn schema_chunk(&self) -> Option<&[u8]> {
        self.chunks.get(0).map(|v| v.as_slice())
    }

    /// Get the data chunk (second chunk)
    pub fn data_chunk(&self) -> Option<&[u8]> {
        self.chunks.get(1).map(|v| v.as_slice())
    }

    /// Get image chunks (all chunks after the first two)
    pub fn image_chunks(&self) -> &[Vec<u8>] {
        if self.chunks.len() > 2 {
            &self.chunks[2..]
        } else {
            &[]
        }
    }
}
