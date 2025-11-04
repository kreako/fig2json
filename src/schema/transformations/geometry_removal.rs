use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove geometry-related fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes geometry fields:
/// - "fillGeometry" - Path commands for fill shapes (M, L, Q, Z, etc.)
/// - "strokeGeometry" - Path commands for stroke shapes
/// - "windingRule" - SVG winding rule property
/// - "styleID" - Internal style reference
///
/// These fields contain detailed path geometry that is overkill for simple
/// shapes in HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all geometry fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_geometry_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "fillGeometry": [
///         {
///             "commands": ["M", 0.0, 0.0, "L", 100.0, 0.0, "Z"],
///             "styleID": 0,
///             "windingRule": {
///                 "__enum__": "WindingRule",
///                 "value": "NONZERO"
///             }
///         }
///     ],
///     "size": {"x": 100.0, "y": 100.0}
/// });
/// remove_geometry_fields(&mut tree).unwrap();
/// // tree now has only "name" and "size" fields
/// ```
pub fn remove_geometry_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove geometry fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove geometry-related fields if they exist
            map.remove("fillGeometry");
            map.remove("strokeGeometry");
            map.remove("windingRule");
            map.remove("styleID");

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
    fn test_remove_fill_geometry() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillGeometry": [
                {
                    "commands": ["M", 0.0, 0.0, "L", 100.0, 0.0, "L", 100.0, 100.0, "Z"],
                    "styleID": 0,
                    "windingRule": {
                        "__enum__": "WindingRule",
                        "value": "NONZERO"
                    }
                }
            ],
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_geometry_fields(&mut tree).unwrap();

        assert!(tree.get("fillGeometry").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_remove_stroke_geometry() {
        let mut tree = json!({
            "name": "Line",
            "strokeGeometry": [
                {
                    "commands": ["M", 0.0, 0.0, "L", 100.0, 100.0],
                    "styleID": 0,
                    "windingRule": {
                        "__enum__": "WindingRule",
                        "value": "NONZERO"
                    }
                }
            ],
            "visible": true
        });

        remove_geometry_fields(&mut tree).unwrap();

        assert!(tree.get("strokeGeometry").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Line"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_both_geometries() {
        let mut tree = json!({
            "name": "Shape",
            "fillGeometry": [
                {
                    "commands": ["M", 0.0, 0.0, "Z"],
                    "styleID": 1
                }
            ],
            "strokeGeometry": [
                {
                    "commands": ["M", 0.0, 0.0, "L", 10.0, 10.0],
                    "styleID": 2
                }
            ],
            "opacity": 1.0
        });

        remove_geometry_fields(&mut tree).unwrap();

        assert!(tree.get("fillGeometry").is_none());
        assert!(tree.get("strokeGeometry").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_remove_nested_geometry() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "fillGeometry": [
                        {
                            "commands": ["M", 0.0, 0.0, "Z"],
                            "styleID": 0
                        }
                    ]
                },
                {
                    "name": "Child2",
                    "strokeGeometry": [
                        {
                            "commands": ["M", 0.0, 0.0, "L", 10.0, 10.0]
                        }
                    ]
                }
            ]
        });

        remove_geometry_fields(&mut tree).unwrap();

        // Children geometries should be removed
        assert!(tree["children"][0].get("fillGeometry").is_none());
        assert_eq!(
            tree["children"][0].get("name").unwrap().as_str(),
            Some("Child1")
        );

        assert!(tree["children"][1].get("strokeGeometry").is_none());
        assert_eq!(
            tree["children"][1].get("name").unwrap().as_str(),
            Some("Child2")
        );
    }

    #[test]
    fn test_remove_winding_rule_standalone() {
        let mut tree = json!({
            "name": "Path",
            "windingRule": {
                "__enum__": "WindingRule",
                "value": "EVENODD"
            },
            "visible": true
        });

        remove_geometry_fields(&mut tree).unwrap();

        assert!(tree.get("windingRule").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Path"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_style_id_standalone() {
        let mut tree = json!({
            "name": "Element",
            "styleID": 42,
            "type": "SHAPE"
        });

        remove_geometry_fields(&mut tree).unwrap();

        assert!(tree.get("styleID").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Element"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("SHAPE"));
    }

    #[test]
    fn test_remove_all_geometry_fields() {
        let mut tree = json!({
            "name": "Complex",
            "fillGeometry": [{"commands": ["M", 0.0, 0.0, "Z"]}],
            "strokeGeometry": [{"commands": ["M", 0.0, 0.0, "L", 10.0, 10.0]}],
            "windingRule": {"__enum__": "WindingRule", "value": "NONZERO"},
            "styleID": 5,
            "opacity": 1.0
        });

        remove_geometry_fields(&mut tree).unwrap();

        assert!(tree.get("fillGeometry").is_none());
        assert!(tree.get("strokeGeometry").is_none());
        assert!(tree.get("windingRule").is_none());
        assert!(tree.get("styleID").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Complex"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_remove_geometry_missing() {
        let mut tree = json!({
            "name": "Simple",
            "x": 10,
            "y": 20,
            "width": 100,
            "height": 100
        });

        remove_geometry_fields(&mut tree).unwrap();

        // Tree without geometry fields should be unchanged
        assert!(tree.get("fillGeometry").is_none());
        assert!(tree.get("strokeGeometry").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Simple"));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));
    }

    #[test]
    fn test_remove_geometry_deeply_nested() {
        let mut tree = json!({
            "document": {
                "fillGeometry": [{"commands": ["M", 0.0, 0.0, "Z"]}],
                "children": [
                    {
                        "children": [
                            {
                                "strokeGeometry": [{"commands": ["L", 10.0, 10.0]}],
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_geometry_fields(&mut tree).unwrap();

        // All geometries should be removed at all levels
        assert!(tree["document"].get("fillGeometry").is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("strokeGeometry")
            .is_none());

        // Other fields should be preserved
        assert_eq!(
            tree["document"]["children"][0]["children"][0]
                .get("name")
                .unwrap()
                .as_str(),
            Some("DeepChild")
        );
    }

    #[test]
    fn test_remove_geometry_empty_object() {
        let mut tree = json!({});

        remove_geometry_fields(&mut tree).unwrap();

        // Empty object should remain empty
        assert_eq!(tree.as_object().unwrap().len(), 0);
    }
}
