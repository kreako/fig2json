use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove document-level property fields from the JSON tree
///
/// Recursively traverses the JSON tree and removes document-level configuration:
/// - "documentColorProfile" - Color profile setting (SRGB, etc.)
///
/// These fields contain document-level metadata that is not needed for
/// basic HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all document property fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_document_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "document": {
///         "name": "Document",
///         "documentColorProfile": {
///             "__enum__": "DocumentColorProfile",
///             "value": "SRGB"
///         }
///     }
/// });
/// remove_document_properties(&mut tree).unwrap();
/// // document now has only "name" field
/// ```
pub fn remove_document_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove document property fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove document property fields if they exist
            map.remove("documentColorProfile");

            // Recurse into all remaining values
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val)?;
                }
            }
        }
        JsonValue::Array(arr) => {
            // Recurse into array elements
            for val in arr.iter_mut() {
                transform_recursive(val)?;
            }
        }
        _ => {
            // Primitives - nothing to do
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_document_color_profile() {
        let mut tree = json!({
            "name": "Document",
            "documentColorProfile": {
                "__enum__": "DocumentColorProfile",
                "value": "SRGB"
            },
            "children": []
        });

        remove_document_properties(&mut tree).unwrap();

        assert!(tree.get("documentColorProfile").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Document"));
        assert!(tree.get("children").is_some());
    }

    #[test]
    fn test_remove_document_color_profile_nested() {
        let mut tree = json!({
            "document": {
                "name": "Document",
                "documentColorProfile": {
                    "__enum__": "DocumentColorProfile",
                    "value": "SRGB"
                },
                "children": [
                    {
                        "name": "Canvas",
                        "documentColorProfile": {
                            "__enum__": "DocumentColorProfile",
                            "value": "DISPLAY_P3"
                        }
                    }
                ]
            }
        });

        remove_document_properties(&mut tree).unwrap();

        // Root document color profile should be removed
        assert!(tree["document"].get("documentColorProfile").is_none());
        assert_eq!(
            tree["document"].get("name").unwrap().as_str(),
            Some("Document")
        );

        // Nested color profile should be removed
        assert!(tree["document"]["children"][0]
            .get("documentColorProfile")
            .is_none());
        assert_eq!(
            tree["document"]["children"][0].get("name").unwrap().as_str(),
            Some("Canvas")
        );
    }

    #[test]
    fn test_no_document_properties() {
        let mut tree = json!({
            "document": {
                "name": "Document",
                "children": []
            }
        });

        remove_document_properties(&mut tree).unwrap();

        // Tree without document properties should be unchanged
        assert_eq!(
            tree["document"].get("name").unwrap().as_str(),
            Some("Document")
        );
        assert!(tree["document"].get("children").is_some());
        assert!(tree["document"].get("documentColorProfile").is_none());
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "document": {
                "name": "Document",
                "documentColorProfile": {
                    "__enum__": "DocumentColorProfile",
                    "value": "SRGB"
                },
                "type": "DOCUMENT",
                "opacity": 1.0,
                "visible": true
            }
        });

        remove_document_properties(&mut tree).unwrap();

        // Only documentColorProfile should be removed
        assert!(tree["document"].get("documentColorProfile").is_none());

        // All other fields preserved
        assert_eq!(
            tree["document"].get("name").unwrap().as_str(),
            Some("Document")
        );
        assert_eq!(
            tree["document"].get("type").unwrap().as_str(),
            Some("DOCUMENT")
        );
        assert_eq!(tree["document"].get("opacity").unwrap().as_f64(), Some(1.0));
        assert_eq!(
            tree["document"].get("visible").unwrap().as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_different_color_profile_values() {
        let mut tree = json!({
            "doc1": {
                "documentColorProfile": {
                    "__enum__": "DocumentColorProfile",
                    "value": "SRGB"
                }
            },
            "doc2": {
                "documentColorProfile": {
                    "__enum__": "DocumentColorProfile",
                    "value": "DISPLAY_P3"
                }
            },
            "doc3": {
                "documentColorProfile": {
                    "__enum__": "DocumentColorProfile",
                    "value": "UNMANAGED"
                }
            }
        });

        remove_document_properties(&mut tree).unwrap();

        // All variations of documentColorProfile should be removed
        assert!(tree["doc1"].get("documentColorProfile").is_none());
        assert!(tree["doc2"].get("documentColorProfile").is_none());
        assert!(tree["doc3"].get("documentColorProfile").is_none());
    }

    #[test]
    fn test_deeply_nested_color_profile() {
        let mut tree = json!({
            "root": {
                "children": [
                    {
                        "children": [
                            {
                                "documentColorProfile": {
                                    "__enum__": "DocumentColorProfile",
                                    "value": "SRGB"
                                },
                                "name": "DeepNode"
                            }
                        ]
                    }
                ]
            }
        });

        remove_document_properties(&mut tree).unwrap();

        // Deeply nested color profile should be removed
        assert!(tree["root"]["children"][0]["children"][0]
            .get("documentColorProfile")
            .is_none());
        assert_eq!(
            tree["root"]["children"][0]["children"][0]
                .get("name")
                .unwrap()
                .as_str(),
            Some("DeepNode")
        );
    }

    #[test]
    fn test_multiple_documents() {
        let mut tree = json!({
            "documents": [
                {
                    "name": "Doc1",
                    "documentColorProfile": {
                        "__enum__": "DocumentColorProfile",
                        "value": "SRGB"
                    }
                },
                {
                    "name": "Doc2",
                    "documentColorProfile": {
                        "__enum__": "DocumentColorProfile",
                        "value": "DISPLAY_P3"
                    }
                }
            ]
        });

        remove_document_properties(&mut tree).unwrap();

        // All color profiles in array should be removed
        assert!(tree["documents"][0].get("documentColorProfile").is_none());
        assert_eq!(
            tree["documents"][0].get("name").unwrap().as_str(),
            Some("Doc1")
        );

        assert!(tree["documents"][1].get("documentColorProfile").is_none());
        assert_eq!(
            tree["documents"][1].get("name").unwrap().as_str(),
            Some("Doc2")
        );
    }

    #[test]
    fn test_empty_object() {
        let mut tree = json!({});

        remove_document_properties(&mut tree).unwrap();

        // Empty object should remain empty
        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_primitives() {
        let mut tree = json!("document");

        remove_document_properties(&mut tree).unwrap();

        // Primitive values should be unchanged
        assert_eq!(tree.as_str(), Some("document"));
    }
}
