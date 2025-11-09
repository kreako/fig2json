use crate::error::Result;
use serde_json::Value as JsonValue;

/// Removes the `type` field from all nodes in the document tree.
///
/// The `type` field indicates the node type (e.g., "FRAME", "INSTANCE", "TEXT", "RECTANGLE").
/// While this provides semantic information about the node structure, it's not necessary
/// for HTML/CSS rendering where element types are typically determined by other properties
/// (e.g., text content, layout properties, visual properties).
///
/// Common type values include:
/// - `FRAME`: Container nodes
/// - `INSTANCE`: Component instances
/// - `TEXT`: Text nodes
/// - `RECTANGLE`: Rectangle shapes
/// - `ELLIPSE`: Ellipse shapes
/// - `VECTOR`: Vector paths
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_type;
///
/// let mut tree = json!({
///     "name": "MyFrame",
///     "type": "FRAME",
///     "size": {"x": 100.0, "y": 100.0}
/// });
///
/// remove_type(&mut tree).unwrap();
///
/// assert!(tree.get("type").is_none());
/// assert!(tree.get("name").is_some());
/// assert!(tree.get("size").is_some());
/// ```
pub fn remove_type(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the type field
            map.remove("type");

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
    fn test_removes_type_from_frame() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_type(&mut tree).unwrap();

        assert!(tree.get("type").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_removes_type_from_instance() {
        let mut tree = json!({
            "name": "Button",
            "type": "INSTANCE",
            "size": {"x": 120.0, "y": 40.0}
        });

        remove_type(&mut tree).unwrap();

        assert!(tree.get("type").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
    }

    #[test]
    fn test_removes_type_from_text() {
        let mut tree = json!({
            "name": "Label",
            "type": "TEXT",
            "fontSize": 14.0
        });

        remove_type(&mut tree).unwrap();

        assert!(tree.get("type").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Label"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_removes_type_from_vector() {
        let mut tree = json!({
            "name": "Icon",
            "type": "VECTOR"
        });

        remove_type(&mut tree).unwrap();

        assert!(tree.get("type").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Icon"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "type": "FRAME",
            "children": [
                {
                    "name": "Child1",
                    "type": "INSTANCE"
                },
                {
                    "name": "Child2",
                    "type": "TEXT"
                }
            ]
        });

        remove_type(&mut tree).unwrap();

        assert!(tree.get("type").is_none());
        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("type").is_none());
        assert!(children[1].get("type").is_none());
        assert_eq!(children[0].get("name").unwrap().as_str(), Some("Child1"));
        assert_eq!(children[1].get("name").unwrap().as_str(), Some("Child2"));
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "type": "FRAME",
            "children": [
                {
                    "name": "Level1",
                    "type": "FRAME",
                    "children": [
                        {
                            "name": "Level2",
                            "type": "RECTANGLE"
                        }
                    ]
                }
            ]
        });

        remove_type(&mut tree).unwrap();

        assert!(tree.get("type").is_none());
        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        assert!(level1.get("type").is_none());
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("type").is_none());
        assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
    }

    #[test]
    fn test_handles_missing_type() {
        let mut tree = json!({
            "name": "Frame",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_type(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_type(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "stackMode": "HORIZONTAL",
            "size": {"x": 100.0, "y": 100.0},
            "transform": {"x": 10.0, "y": 20.0},
            "fillPaints": [{"color": "#ffffff", "type": "SOLID"}]
        });

        remove_type(&mut tree).unwrap();

        assert!(tree.get("type").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("stackMode").unwrap().as_str(), Some("HORIZONTAL"));
        assert!(tree.get("size").is_some());
        assert!(tree.get("transform").is_some());
        assert!(tree.get("fillPaints").is_some());

        // Note: type is removed recursively, including from nested paint objects
        let paints = tree.get("fillPaints").unwrap().as_array().unwrap();
        assert!(paints[0].get("type").is_none());
        assert_eq!(paints[0].get("color").unwrap().as_str(), Some("#ffffff"));
    }

    #[test]
    fn test_handles_multiple_types_in_array() {
        let mut tree = json!({
            "children": [
                {"name": "A", "type": "FRAME"},
                {"name": "B", "type": "TEXT"},
                {"name": "C", "type": "RECTANGLE"},
                {"name": "D", "type": "INSTANCE"},
                {"name": "E"}
            ]
        });

        remove_type(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        for child in children {
            assert!(child.get("type").is_none());
            assert!(child.get("name").is_some());
        }
    }

    #[test]
    fn test_removes_all_node_types() {
        let mut tree = json!({
            "children": [
                {"name": "Frame", "type": "FRAME"},
                {"name": "Instance", "type": "INSTANCE"},
                {"name": "Text", "type": "TEXT"},
                {"name": "Rectangle", "type": "RECTANGLE"},
                {"name": "Ellipse", "type": "ELLIPSE"},
                {"name": "Vector", "type": "VECTOR"},
                {"name": "Group", "type": "GROUP"}
            ]
        });

        remove_type(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        for child in children {
            assert!(child.get("type").is_none());
        }
    }
}
