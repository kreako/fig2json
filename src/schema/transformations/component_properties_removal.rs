use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove componentPropAssignments fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes all "componentPropAssignments" fields.
/// These fields contain Figma component property system metadata with complex nested
/// structures that are not needed for HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all componentPropAssignments fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_component_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Button",
///     "componentPropAssignments": [
///         {
///             "defID": {"localID": 51, "sessionID": 488},
///             "value": {"textValue": {"characters": "Click me"}}
///         }
///     ],
///     "visible": true
/// });
/// remove_component_properties(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_component_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove componentPropAssignments fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the "componentPropAssignments" field if it exists
            map.remove("componentPropAssignments");

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
    fn test_remove_component_prop_assignments_simple() {
        let mut tree = json!({
            "name": "Button",
            "componentPropAssignments": [
                {
                    "defID": {"localID": 51, "sessionID": 488},
                    "value": {"textValue": {"characters": "Click me"}}
                }
            ],
            "visible": true
        });

        remove_component_properties(&mut tree).unwrap();

        assert!(tree.get("componentPropAssignments").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_component_prop_assignments_complex() {
        let mut tree = json!({
            "name": "ComplexButton",
            "componentPropAssignments": [
                {
                    "defID": {"localID": 51, "sessionID": 488},
                    "value": {
                        "textValue": {
                            "characters": "Add new role",
                            "lines": [
                                {
                                    "indentationLevel": 0,
                                    "isFirstLineOfList": false,
                                    "lineType": "PLAIN",
                                    "styleId": 0
                                }
                            ]
                        }
                    },
                    "varValue": {
                        "dataType": "TEXT_DATA",
                        "value": {
                            "textDataValue": {
                                "characters": "Add new role"
                            }
                        }
                    }
                }
            ]
        });

        remove_component_properties(&mut tree).unwrap();

        assert!(tree.get("componentPropAssignments").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("ComplexButton"));
    }

    #[test]
    fn test_remove_component_prop_assignments_nested() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Button1",
                    "componentPropAssignments": [
                        {"defID": {"localID": 1, "sessionID": 1}}
                    ]
                },
                {
                    "name": "Button2",
                    "componentPropAssignments": [
                        {"defID": {"localID": 2, "sessionID": 1}}
                    ]
                }
            ]
        });

        remove_component_properties(&mut tree).unwrap();

        assert!(tree["children"][0].get("componentPropAssignments").is_none());
        assert!(tree["children"][1].get("componentPropAssignments").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Button1"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Button2"));
    }

    #[test]
    fn test_remove_component_prop_assignments_deeply_nested() {
        let mut tree = json!({
            "document": {
                "componentPropAssignments": [],
                "children": [
                    {
                        "componentPropAssignments": [
                            {"defID": {"localID": 1, "sessionID": 1}}
                        ],
                        "children": [
                            {
                                "componentPropAssignments": [
                                    {"defID": {"localID": 2, "sessionID": 1}}
                                ],
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_component_properties(&mut tree).unwrap();

        assert!(tree["document"].get("componentPropAssignments").is_none());
        assert!(tree["document"]["children"][0]
            .get("componentPropAssignments")
            .is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("componentPropAssignments")
            .is_none());
    }

    #[test]
    fn test_remove_component_prop_assignments_missing() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "visible": true
        });

        remove_component_properties(&mut tree).unwrap();

        assert!(tree.get("componentPropAssignments").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_remove_component_prop_assignments_empty_array() {
        let mut tree = json!({
            "name": "Component",
            "componentPropAssignments": [],
            "visible": true
        });

        remove_component_properties(&mut tree).unwrap();

        assert!(tree.get("componentPropAssignments").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Component"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_component_prop_assignments_preserves_other_fields() {
        let mut tree = json!({
            "name": "Button",
            "componentPropAssignments": [
                {
                    "defID": {"localID": 51, "sessionID": 488},
                    "value": {"textValue": {"characters": "Submit"}}
                }
            ],
            "cornerRadius": 12.0,
            "fillPaints": [{"color": "#343439", "type": "SOLID"}],
            "size": {"x": 327.0, "y": 48.0},
            "opacity": 1.0
        });

        remove_component_properties(&mut tree).unwrap();

        assert!(tree.get("componentPropAssignments").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
        assert_eq!(tree.get("cornerRadius").unwrap().as_f64(), Some(12.0));
        assert!(tree.get("fillPaints").is_some());
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_remove_component_prop_assignments_multiple_props() {
        let mut tree = json!({
            "name": "MultiPropComponent",
            "componentPropAssignments": [
                {
                    "defID": {"localID": 1, "sessionID": 1},
                    "value": {"textValue": {"characters": "Text 1"}}
                },
                {
                    "defID": {"localID": 2, "sessionID": 1},
                    "value": {"boolValue": true}
                },
                {
                    "defID": {"localID": 3, "sessionID": 1},
                    "value": {"numberValue": 42.0}
                }
            ]
        });

        remove_component_properties(&mut tree).unwrap();

        assert!(tree.get("componentPropAssignments").is_none());
        assert_eq!(
            tree.get("name").unwrap().as_str(),
            Some("MultiPropComponent")
        );
    }

    #[test]
    fn test_remove_component_prop_assignments_in_arrays() {
        let mut tree = json!({
            "components": [
                {
                    "name": "Comp1",
                    "componentPropAssignments": [
                        {"defID": {"localID": 1, "sessionID": 1}}
                    ]
                },
                {
                    "name": "Comp2",
                    "componentPropAssignments": []
                }
            ]
        });

        remove_component_properties(&mut tree).unwrap();

        assert!(tree["components"][0].get("componentPropAssignments").is_none());
        assert!(tree["components"][1].get("componentPropAssignments").is_none());
        assert_eq!(tree["components"][0]["name"].as_str(), Some("Comp1"));
        assert_eq!(tree["components"][1]["name"].as_str(), Some("Comp2"));
    }

    #[test]
    fn test_remove_component_prop_assignments_empty_object() {
        let mut tree = json!({});

        remove_component_properties(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }
}
