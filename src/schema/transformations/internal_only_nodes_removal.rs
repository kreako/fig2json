use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove internal-only nodes from the JSON tree
///
/// Recursively traverses the JSON tree and filters out nodes that have
/// `internalOnly: true`. These are Figma internal nodes that are not meant
/// for rendering and should not be included in the final output.
///
/// Internal-only nodes are typically removed from "children" arrays.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all internal-only nodes
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_internal_only_nodes;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "children": [
///         {"name": "Visible", "visible": true},
///         {"name": "Internal", "internalOnly": true}
///     ]
/// });
/// remove_internal_only_nodes(&mut tree).unwrap();
/// // children array now contains only the "Visible" node
/// ```
pub fn remove_internal_only_nodes(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove internal-only nodes from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Recurse into all values
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val)?;
                }
            }

            // After recursion, remove internalOnly field itself
            // (it was only used for filtering, not needed in output)
            map.remove("internalOnly");
        }
        JsonValue::Array(arr) => {
            // Filter out nodes with internalOnly: true FIRST (before recursing)
            arr.retain(|node| {
                if let Some(obj) = node.as_object() {
                    // Keep node if internalOnly is not true
                    !obj.get("internalOnly")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                } else {
                    // Keep non-object values
                    true
                }
            });

            // Then recurse into remaining array elements
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
    fn test_remove_internal_only_node() {
        let mut tree = json!({
            "children": [
                {"name": "Visible", "visible": true},
                {"name": "Internal", "internalOnly": true, "visible": false}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0]["name"].as_str(), Some("Visible"));
    }

    #[test]
    fn test_preserve_visible_nodes() {
        let mut tree = json!({
            "children": [
                {"name": "Node1", "visible": true},
                {"name": "Node2", "visible": true},
                {"name": "Node3", "visible": false}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        // All nodes without internalOnly should be preserved
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn test_remove_multiple_internal_nodes() {
        let mut tree = json!({
            "children": [
                {"name": "Visible1", "visible": true},
                {"name": "Internal1", "internalOnly": true},
                {"name": "Visible2", "visible": true},
                {"name": "Internal2", "internalOnly": true}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0]["name"].as_str(), Some("Visible1"));
        assert_eq!(children[1]["name"].as_str(), Some("Visible2"));
    }

    #[test]
    fn test_all_internal_nodes() {
        let mut tree = json!({
            "children": [
                {"name": "Internal1", "internalOnly": true},
                {"name": "Internal2", "internalOnly": true}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        // All nodes removed, empty array
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn test_no_internal_nodes() {
        let mut tree = json!({
            "children": [
                {"name": "Node1"},
                {"name": "Node2"}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        // All nodes preserved
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_nested_children() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Parent",
                    "children": [
                        {"name": "Child1", "visible": true},
                        {"name": "Internal", "internalOnly": true}
                    ]
                }
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let parent_children = tree["children"][0]["children"].as_array().unwrap();
        assert_eq!(parent_children.len(), 1);
        assert_eq!(parent_children[0]["name"].as_str(), Some("Child1"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "name": "Canvas",
                        "children": [
                            {"name": "Frame", "visible": true},
                            {"name": "Internal Canvas", "internalOnly": true}
                        ]
                    }
                ]
            }
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let canvas_children = tree["document"]["children"][0]["children"]
            .as_array()
            .unwrap();
        assert_eq!(canvas_children.len(), 1);
        assert_eq!(canvas_children[0]["name"].as_str(), Some("Frame"));
    }

    #[test]
    fn test_internal_only_false() {
        let mut tree = json!({
            "children": [
                {"name": "Node1", "internalOnly": false},
                {"name": "Node2", "internalOnly": true}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        // Only internalOnly: true should be filtered
        assert_eq!(children.len(), 1);
        assert_eq!(children[0]["name"].as_str(), Some("Node1"));
        // internalOnly field should be removed from kept nodes
        assert!(children[0].get("internalOnly").is_none());
    }

    #[test]
    fn test_remove_internal_only_field() {
        let mut tree = json!({
            "name": "Node",
            "internalOnly": false,
            "visible": true
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        // internalOnly field should be removed even if false
        assert!(tree.get("internalOnly").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_non_object_array_elements() {
        let mut tree = json!({
            "data": [1, 2, 3, "string"]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let data = tree.get("data").unwrap().as_array().unwrap();
        // Non-object elements should be preserved
        assert_eq!(data.len(), 4);
    }
}
