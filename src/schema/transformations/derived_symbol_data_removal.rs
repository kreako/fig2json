use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove derived symbol data fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes Figma-computed component metadata:
/// - "derivedSymbolData" - Computed component instance data with overrides
/// - "derivedSymbolDataLayoutVersion" - Version number for derived symbol data
///
/// These fields contain Figma-specific computed metadata that are not needed
/// for HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all derived symbol data fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_derived_symbol_data;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Component",
///     "derivedSymbolData": [
///         {
///             "guidPath": {
///                 "guids": [{"localID": 1, "sessionID": 1}]
///             },
///             "size": {"x": 100.0, "y": 50.0}
///         }
///     ],
///     "derivedSymbolDataLayoutVersion": 1,
///     "visible": true
/// });
/// remove_derived_symbol_data(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_derived_symbol_data(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove derived symbol data fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the derived symbol data fields if they exist
            map.remove("derivedSymbolData");
            map.remove("derivedSymbolDataLayoutVersion");

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
    fn test_remove_derived_symbol_data_simple() {
        let mut tree = json!({
            "name": "Component",
            "derivedSymbolData": [
                {
                    "guidPath": {
                        "guids": [{"localID": 1, "sessionID": 1}]
                    },
                    "size": {"x": 100.0, "y": 50.0}
                }
            ],
            "derivedSymbolDataLayoutVersion": 1,
            "visible": true
        });

        remove_derived_symbol_data(&mut tree).unwrap();

        assert!(tree.get("derivedSymbolData").is_none());
        assert!(tree.get("derivedSymbolDataLayoutVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Component"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_derived_symbol_data_complex() {
        let mut tree = json!({
            "name": "Instance",
            "derivedSymbolData": [
                {
                    "guidPath": {
                        "guids": [
                            {"localID": 1, "sessionID": 1},
                            {"localID": 2, "sessionID": 1}
                        ]
                    },
                    "size": {"x": 200.0, "y": 100.0},
                    "transform": {"x": 10.0, "y": 20.0}
                },
                {
                    "guidPath": {
                        "guids": [{"localID": 3, "sessionID": 1}]
                    }
                }
            ],
            "derivedSymbolDataLayoutVersion": 2
        });

        remove_derived_symbol_data(&mut tree).unwrap();

        assert!(tree.get("derivedSymbolData").is_none());
        assert!(tree.get("derivedSymbolDataLayoutVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Instance"));
    }

    #[test]
    fn test_remove_derived_symbol_data_nested() {
        let mut tree = json!({
            "name": "Root",
            "derivedSymbolData": [],
            "derivedSymbolDataLayoutVersion": 1,
            "children": [
                {
                    "name": "Child1",
                    "derivedSymbolData": [
                        {
                            "guidPath": {
                                "guids": [{"localID": 1, "sessionID": 1}]
                            }
                        }
                    ],
                    "derivedSymbolDataLayoutVersion": 1
                },
                {
                    "name": "Child2",
                    "derivedSymbolDataLayoutVersion": 2
                }
            ]
        });

        remove_derived_symbol_data(&mut tree).unwrap();

        assert!(tree.get("derivedSymbolData").is_none());
        assert!(tree.get("derivedSymbolDataLayoutVersion").is_none());
        assert!(tree["children"][0].get("derivedSymbolData").is_none());
        assert!(tree["children"][0]
            .get("derivedSymbolDataLayoutVersion")
            .is_none());
        assert!(tree["children"][1]
            .get("derivedSymbolDataLayoutVersion")
            .is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Root"));
    }

    #[test]
    fn test_remove_derived_symbol_data_only_version() {
        let mut tree = json!({
            "name": "Node",
            "derivedSymbolDataLayoutVersion": 1,
            "visible": true
        });

        remove_derived_symbol_data(&mut tree).unwrap();

        assert!(tree.get("derivedSymbolDataLayoutVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_derived_symbol_data_only_array() {
        let mut tree = json!({
            "name": "Node",
            "derivedSymbolData": [
                {"guidPath": {"guids": []}}
            ],
            "visible": true
        });

        remove_derived_symbol_data(&mut tree).unwrap();

        assert!(tree.get("derivedSymbolData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_derived_symbol_data_missing() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "visible": true
        });

        remove_derived_symbol_data(&mut tree).unwrap();

        assert!(tree.get("derivedSymbolData").is_none());
        assert!(tree.get("derivedSymbolDataLayoutVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_remove_derived_symbol_data_preserves_other_fields() {
        let mut tree = json!({
            "name": "Component",
            "derivedSymbolData": [],
            "derivedSymbolDataLayoutVersion": 1,
            "size": {"x": 100, "y": 200},
            "transform": {"x": 10, "y": 20},
            "opacity": 0.9
        });

        remove_derived_symbol_data(&mut tree).unwrap();

        assert!(tree.get("derivedSymbolData").is_none());
        assert!(tree.get("derivedSymbolDataLayoutVersion").is_none());
        assert_eq!(tree["size"]["x"].as_i64(), Some(100));
        assert_eq!(tree["transform"]["x"].as_i64(), Some(10));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.9));
    }

    #[test]
    fn test_remove_derived_symbol_data_in_arrays() {
        let mut tree = json!({
            "components": [
                {
                    "name": "Comp1",
                    "derivedSymbolData": [
                        {"guidPath": {"guids": []}}
                    ],
                    "derivedSymbolDataLayoutVersion": 1
                },
                {
                    "name": "Comp2",
                    "derivedSymbolDataLayoutVersion": 2
                }
            ]
        });

        remove_derived_symbol_data(&mut tree).unwrap();

        assert!(tree["components"][0].get("derivedSymbolData").is_none());
        assert!(tree["components"][0]
            .get("derivedSymbolDataLayoutVersion")
            .is_none());
        assert!(tree["components"][1]
            .get("derivedSymbolDataLayoutVersion")
            .is_none());
        assert_eq!(
            tree["components"][0].get("name").unwrap().as_str(),
            Some("Comp1")
        );
    }

    #[test]
    fn test_remove_derived_symbol_data_empty_array() {
        let mut tree = json!({
            "name": "Node",
            "derivedSymbolData": [],
            "derivedSymbolDataLayoutVersion": 1
        });

        remove_derived_symbol_data(&mut tree).unwrap();

        assert!(tree.get("derivedSymbolData").is_none());
        assert!(tree.get("derivedSymbolDataLayoutVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
    }

    #[test]
    fn test_remove_derived_symbol_data_deeply_nested() {
        let mut tree = json!({
            "document": {
                "derivedSymbolData": [],
                "derivedSymbolDataLayoutVersion": 1,
                "children": [
                    {
                        "derivedSymbolData": [],
                        "children": [
                            {
                                "derivedSymbolDataLayoutVersion": 1,
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_derived_symbol_data(&mut tree).unwrap();

        assert!(tree["document"].get("derivedSymbolData").is_none());
        assert!(tree["document"]
            .get("derivedSymbolDataLayoutVersion")
            .is_none());
        assert!(tree["document"]["children"][0]
            .get("derivedSymbolData")
            .is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("derivedSymbolDataLayoutVersion")
            .is_none());
    }
}
