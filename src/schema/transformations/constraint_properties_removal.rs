use crate::error::Result;
use serde_json::Value as JsonValue;

/// Removes Figma-specific constraint properties from the document tree.
///
/// This transformation removes the following fields:
/// - `horizontalConstraint`: Controls how nodes resize horizontally in Figma's auto-layout
/// - `verticalConstraint`: Controls how nodes resize vertically in Figma's auto-layout
///
/// These properties are specific to Figma's constraint system and are not directly
/// needed for HTML/CSS rendering, as CSS uses different mechanisms (flexbox, grid, etc.)
/// for responsive layout behavior.
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_constraint_properties;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "horizontalConstraint": "CENTER",
///     "verticalConstraint": "SCALE",
///     "size": {"x": 100.0, "y": 100.0}
/// });
///
/// remove_constraint_properties(&mut tree).unwrap();
///
/// assert!(tree.get("horizontalConstraint").is_none());
/// assert!(tree.get("verticalConstraint").is_none());
/// assert!(tree.get("size").is_some());
/// ```
pub fn remove_constraint_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove constraint properties
            map.remove("horizontalConstraint");
            map.remove("verticalConstraint");

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
    fn test_removes_horizontal_constraint() {
        let mut tree = json!({
            "name": "Frame",
            "horizontalConstraint": "CENTER",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_constraint_properties(&mut tree).unwrap();

        assert!(tree.get("horizontalConstraint").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_removes_vertical_constraint() {
        let mut tree = json!({
            "name": "Frame",
            "verticalConstraint": "SCALE",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_constraint_properties(&mut tree).unwrap();

        assert!(tree.get("verticalConstraint").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_removes_both_constraints() {
        let mut tree = json!({
            "name": "Frame",
            "horizontalConstraint": "CENTER",
            "verticalConstraint": "MAX",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_constraint_properties(&mut tree).unwrap();

        assert!(tree.get("horizontalConstraint").is_none());
        assert!(tree.get("verticalConstraint").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "children": [
                {
                    "name": "Child1",
                    "horizontalConstraint": "LEFT",
                    "verticalConstraint": "TOP"
                },
                {
                    "name": "Child2",
                    "horizontalConstraint": "RIGHT"
                }
            ]
        });

        remove_constraint_properties(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("horizontalConstraint").is_none());
        assert!(children[0].get("verticalConstraint").is_none());
        assert!(children[1].get("horizontalConstraint").is_none());
        assert_eq!(children[0].get("name").unwrap().as_str(), Some("Child1"));
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "horizontalConstraint": "SCALE",
            "children": [
                {
                    "name": "Level1",
                    "verticalConstraint": "CENTER",
                    "children": [
                        {
                            "name": "Level2",
                            "horizontalConstraint": "MIN",
                            "verticalConstraint": "MAX"
                        }
                    ]
                }
            ]
        });

        remove_constraint_properties(&mut tree).unwrap();

        assert!(tree.get("horizontalConstraint").is_none());
        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        assert!(level1.get("verticalConstraint").is_none());
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("horizontalConstraint").is_none());
        assert!(level2.get("verticalConstraint").is_none());
        assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
    }

    #[test]
    fn test_handles_missing_constraint_properties() {
        let mut tree = json!({
            "name": "Frame",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_constraint_properties(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_constraint_properties(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "horizontalConstraint": "CENTER",
            "verticalConstraint": "SCALE",
            "stackMode": "HORIZONTAL",
            "size": {"x": 100.0, "y": 100.0},
            "transform": {"x": 10.0, "y": 20.0}
        });

        remove_constraint_properties(&mut tree).unwrap();

        assert!(tree.get("horizontalConstraint").is_none());
        assert!(tree.get("verticalConstraint").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("stackMode").unwrap().as_str(), Some("HORIZONTAL"));
        assert!(tree.get("size").is_some());
        assert!(tree.get("transform").is_some());
    }

    #[test]
    fn test_handles_multiple_occurrences_in_array() {
        let mut tree = json!({
            "children": [
                {"name": "A", "horizontalConstraint": "LEFT"},
                {"name": "B", "verticalConstraint": "TOP"},
                {"name": "C", "horizontalConstraint": "RIGHT", "verticalConstraint": "BOTTOM"},
                {"name": "D"}
            ]
        });

        remove_constraint_properties(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        for child in children {
            assert!(child.get("horizontalConstraint").is_none());
            assert!(child.get("verticalConstraint").is_none());
            assert!(child.get("name").is_some());
        }
    }
}
