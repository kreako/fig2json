use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove the root-level "blobs" field from the JSON document
///
/// After blob substitution, the blobs have been integrated into the document tree,
/// so the separate blobs array is no longer needed. This function removes it from
/// the root-level JSON object.
///
/// # Arguments
/// * `json` - The root JSON object (typically contains version, fileType, document, blobs)
///
/// # Returns
/// * `Ok(())` - Successfully removed the blobs field (or it didn't exist)
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_root_blobs;
/// use serde_json::json;
///
/// let mut output = json!({
///     "version": 48,
///     "fileType": "figma",
///     "document": {"name": "Root"},
///     "blobs": [{"bytes": "..."}]
/// });
/// remove_root_blobs(&mut output).unwrap();
/// // output now only has version, fileType, and document
/// ```
pub fn remove_root_blobs(json: &mut JsonValue) -> Result<()> {
    if let Some(obj) = json.as_object_mut() {
        obj.remove("blobs");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_root_blobs() {
        let mut output = json!({
            "version": 48,
            "fileType": "figma",
            "document": {
                "name": "Root",
                "children": []
            },
            "blobs": [
                {"bytes": "SGVsbG8="},
                {"bytes": "V29ybGQ="}
            ]
        });

        remove_root_blobs(&mut output).unwrap();

        // Blobs field should be removed
        assert!(output.get("blobs").is_none());

        // Other fields should be preserved
        assert_eq!(output.get("version").unwrap().as_i64(), Some(48));
        assert_eq!(output.get("fileType").unwrap().as_str(), Some("figma"));
        assert!(output.get("document").is_some());
        assert_eq!(
            output.get("document").unwrap().get("name").unwrap().as_str(),
            Some("Root")
        );
    }

    #[test]
    fn test_remove_root_blobs_already_missing() {
        let mut output = json!({
            "version": 101,
            "fileType": "figjam",
            "document": {
                "name": "Board"
            }
        });

        // Should not fail if blobs is already missing
        remove_root_blobs(&mut output).unwrap();

        assert!(output.get("blobs").is_none());
        assert_eq!(output.get("version").unwrap().as_i64(), Some(101));
    }

    #[test]
    fn test_remove_root_blobs_preserves_all_fields() {
        let mut output = json!({
            "version": 50,
            "fileType": "figma",
            "document": {
                "name": "Document",
                "children": [
                    {"name": "Child1"},
                    {"name": "Child2"}
                ]
            },
            "blobs": [],
            "metadata": {
                "custom": "field"
            }
        });

        remove_root_blobs(&mut output).unwrap();

        // Only blobs should be removed
        assert!(output.get("blobs").is_none());

        // All other fields preserved
        assert_eq!(output.get("version").unwrap().as_i64(), Some(50));
        assert_eq!(output.get("fileType").unwrap().as_str(), Some("figma"));
        assert!(output.get("document").is_some());
        assert!(output.get("metadata").is_some());
        assert_eq!(
            output.get("metadata").unwrap().get("custom").unwrap().as_str(),
            Some("field")
        );
    }

    #[test]
    fn test_remove_root_blobs_not_an_object() {
        let mut output = json!([1, 2, 3]);

        // Should not fail on non-object input
        remove_root_blobs(&mut output).unwrap();

        // Array should remain unchanged
        assert_eq!(output.as_array().unwrap().len(), 3);
    }
}
