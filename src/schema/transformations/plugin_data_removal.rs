use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove pluginData fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes all "pluginData" fields.
/// These fields contain Figma plugin storage data that is not needed for
/// HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all pluginData fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_plugin_data;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Node",
///     "pluginData": [],
///     "visible": true
/// });
/// remove_plugin_data(&mut tree).unwrap();
/// // tree now has only "name" and "visible" fields
/// ```
pub fn remove_plugin_data(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove pluginData fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the "pluginData" field if it exists
            map.remove("pluginData");

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
    fn test_remove_plugin_data_empty_array() {
        let mut tree = json!({
            "name": "Node",
            "pluginData": [],
            "visible": true
        });

        remove_plugin_data(&mut tree).unwrap();

        assert!(tree.get("pluginData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_plugin_data_with_content() {
        let mut tree = json!({
            "name": "Icon",
            "pluginData": [
                {
                    "pluginID": "some-plugin-id",
                    "data": {"key": "value"}
                }
            ],
            "visible": true
        });

        remove_plugin_data(&mut tree).unwrap();

        assert!(tree.get("pluginData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Icon"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_plugin_data_nested() {
        let mut tree = json!({
            "name": "Root",
            "pluginData": [],
            "children": [
                {
                    "name": "Child1",
                    "pluginData": [
                        {"pluginID": "plugin1", "data": {}}
                    ]
                },
                {
                    "name": "Child2",
                    "pluginData": []
                }
            ]
        });

        remove_plugin_data(&mut tree).unwrap();

        assert!(tree.get("pluginData").is_none());
        assert!(tree["children"][0].get("pluginData").is_none());
        assert!(tree["children"][1].get("pluginData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Root"));
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));
    }

    #[test]
    fn test_remove_plugin_data_deeply_nested() {
        let mut tree = json!({
            "document": {
                "pluginData": [],
                "children": [
                    {
                        "pluginData": [{"pluginID": "test"}],
                        "children": [
                            {
                                "pluginData": [],
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_plugin_data(&mut tree).unwrap();

        assert!(tree["document"].get("pluginData").is_none());
        assert!(tree["document"]["children"][0].get("pluginData").is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("pluginData")
            .is_none());
    }

    #[test]
    fn test_remove_plugin_data_missing() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "visible": true
        });

        remove_plugin_data(&mut tree).unwrap();

        assert!(tree.get("pluginData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_remove_plugin_data_preserves_other_fields() {
        let mut tree = json!({
            "name": "Node",
            "pluginData": [
                {
                    "pluginID": "my-plugin",
                    "data": {
                        "customProperty": "customValue"
                    }
                }
            ],
            "size": {"x": 100, "y": 200},
            "opacity": 0.9,
            "visible": true
        });

        remove_plugin_data(&mut tree).unwrap();

        assert!(tree.get("pluginData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
        assert!(tree.get("size").is_some());
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.9));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_plugin_data_in_arrays() {
        let mut tree = json!({
            "nodes": [
                {
                    "pluginData": [],
                    "name": "Node1"
                },
                {
                    "pluginData": [{"pluginID": "test"}],
                    "name": "Node2"
                }
            ]
        });

        remove_plugin_data(&mut tree).unwrap();

        assert!(tree["nodes"][0].get("pluginData").is_none());
        assert_eq!(
            tree["nodes"][0].get("name").unwrap().as_str(),
            Some("Node1")
        );
        assert!(tree["nodes"][1].get("pluginData").is_none());
        assert_eq!(
            tree["nodes"][1].get("name").unwrap().as_str(),
            Some("Node2")
        );
    }

    #[test]
    fn test_remove_plugin_data_empty_object() {
        let mut tree = json!({});

        remove_plugin_data(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_remove_plugin_data_primitives() {
        let mut tree = json!(123);

        remove_plugin_data(&mut tree).unwrap();

        assert_eq!(tree.as_i64(), Some(123));
    }

    #[test]
    fn test_remove_plugin_data_mixed_nodes() {
        let mut tree = json!({
            "children": [
                {
                    "name": "WithPluginData",
                    "pluginData": [{"pluginID": "plugin1"}]
                },
                {
                    "name": "WithoutPluginData"
                },
                {
                    "name": "AlsoWithPluginData",
                    "pluginData": []
                }
            ]
        });

        remove_plugin_data(&mut tree).unwrap();

        assert!(tree["children"][0].get("pluginData").is_none());
        assert!(tree["children"][1].get("pluginData").is_none());
        assert!(tree["children"][2].get("pluginData").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("WithPluginData"));
        assert_eq!(
            tree["children"][1]["name"].as_str(),
            Some("WithoutPluginData")
        );
        assert_eq!(
            tree["children"][2]["name"].as_str(),
            Some("AlsoWithPluginData")
        );
    }
}
