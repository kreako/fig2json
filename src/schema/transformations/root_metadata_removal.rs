use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove root-level metadata fields (version and fileType) from the JSON document
///
/// These fields are Figma-specific metadata that are not needed for HTML/CSS rendering.
/// This function removes them only from the root level to keep the output clean and
/// focused on renderable content.
///
/// Removed fields:
/// - "version" - The Figma file format version number
/// - "fileType" - The file type identifier (e.g., "figma", "figjam")
///
/// # Arguments
/// * `json` - The root JSON object (typically contains version, fileType, document)
///
/// # Returns
/// * `Ok(())` - Successfully removed the metadata fields (or they didn't exist)
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_root_metadata;
/// use serde_json::json;
///
/// let mut output = json!({
///     "version": 101,
///     "fileType": "figma",
///     "document": {"name": "Root"}
/// });
/// remove_root_metadata(&mut output).unwrap();
/// // output now only has document
/// ```
pub fn remove_root_metadata(json: &mut JsonValue) -> Result<()> {
    if let Some(obj) = json.as_object_mut() {
        obj.remove("version");
        obj.remove("fileType");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_version_and_file_type() {
        let mut output = json!({
            "version": 101,
            "fileType": "figma",
            "document": {
                "name": "Root",
                "children": []
            }
        });

        remove_root_metadata(&mut output).unwrap();

        // version and fileType should be removed
        assert!(output.get("version").is_none());
        assert!(output.get("fileType").is_none());

        // document should be preserved
        assert!(output.get("document").is_some());
        assert_eq!(
            output.get("document").unwrap().get("name").unwrap().as_str(),
            Some("Root")
        );
    }

    #[test]
    fn test_remove_only_version() {
        let mut output = json!({
            "version": 101,
            "document": {
                "name": "Root"
            }
        });

        remove_root_metadata(&mut output).unwrap();

        // version should be removed
        assert!(output.get("version").is_none());

        // document should be preserved
        assert!(output.get("document").is_some());
    }

    #[test]
    fn test_remove_only_file_type() {
        let mut output = json!({
            "fileType": "figjam",
            "document": {
                "name": "Board"
            }
        });

        remove_root_metadata(&mut output).unwrap();

        // fileType should be removed
        assert!(output.get("fileType").is_none());

        // document should be preserved
        assert!(output.get("document").is_some());
    }

    #[test]
    fn test_already_missing() {
        let mut output = json!({
            "document": {
                "name": "Document"
            }
        });

        // Should not fail if version and fileType are already missing
        remove_root_metadata(&mut output).unwrap();

        assert!(output.get("version").is_none());
        assert!(output.get("fileType").is_none());
        assert!(output.get("document").is_some());
    }

    #[test]
    fn test_preserves_other_root_fields() {
        let mut output = json!({
            "version": 101,
            "fileType": "figma",
            "document": {
                "name": "Document"
            },
            "metadata": {
                "custom": "field"
            },
            "otherData": [1, 2, 3]
        });

        remove_root_metadata(&mut output).unwrap();

        // Only version and fileType should be removed
        assert!(output.get("version").is_none());
        assert!(output.get("fileType").is_none());

        // All other fields preserved
        assert!(output.get("document").is_some());
        assert!(output.get("metadata").is_some());
        assert!(output.get("otherData").is_some());
        assert_eq!(
            output.get("metadata").unwrap().get("custom").unwrap().as_str(),
            Some("field")
        );
    }

    #[test]
    fn test_preserves_nested_version_and_file_type() {
        let mut output = json!({
            "version": 101,
            "fileType": "figma",
            "document": {
                "name": "Document",
                "version": 2,
                "metadata": {
                    "fileType": "custom"
                }
            }
        });

        remove_root_metadata(&mut output).unwrap();

        // Root-level version and fileType removed
        assert!(output.get("version").is_none());
        assert!(output.get("fileType").is_none());

        // Nested version and fileType preserved
        assert_eq!(
            output["document"].get("version").unwrap().as_i64(),
            Some(2)
        );
        assert_eq!(
            output["document"]["metadata"]
                .get("fileType")
                .unwrap()
                .as_str(),
            Some("custom")
        );
    }

    #[test]
    fn test_different_version_values() {
        let versions = vec![48, 50, 101, 999];

        for version in versions {
            let mut output = json!({
                "version": version,
                "document": {}
            });

            remove_root_metadata(&mut output).unwrap();

            // All version values should be removed
            assert!(output.get("version").is_none());
        }
    }

    #[test]
    fn test_different_file_types() {
        let file_types = vec!["figma", "figjam", "whiteboard"];

        for file_type in file_types {
            let mut output = json!({
                "fileType": file_type,
                "document": {}
            });

            remove_root_metadata(&mut output).unwrap();

            // All fileType values should be removed
            assert!(output.get("fileType").is_none());
        }
    }

    #[test]
    fn test_not_an_object() {
        let mut output = json!([1, 2, 3]);

        // Should not fail on non-object input
        remove_root_metadata(&mut output).unwrap();

        // Array should remain unchanged
        assert_eq!(output.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_empty_object() {
        let mut output = json!({});

        remove_root_metadata(&mut output).unwrap();

        // Empty object should remain empty
        assert_eq!(output.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_string_primitive() {
        let mut output = json!("document");

        // Should not fail on primitive input
        remove_root_metadata(&mut output).unwrap();

        // String should remain unchanged
        assert_eq!(output.as_str(), Some("document"));
    }
}
