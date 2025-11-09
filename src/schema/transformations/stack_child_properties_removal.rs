use crate::error::Result;
use serde_json::Value as JsonValue;

/// Removes Figma-specific stack child properties from the document tree.
///
/// This transformation removes the following fields:
/// - `stackChildAlignSelf`: Controls how an individual child aligns within its parent's auto-layout
/// - `stackChildPrimaryGrow`: Controls whether a child grows to fill available space in the primary axis
///
/// These properties are specific to Figma's auto-layout child configuration and are not directly
/// needed for HTML/CSS rendering. CSS uses different mechanisms (flexbox `align-self`, `flex-grow`, etc.)
/// for similar behavior, but the mapping is not always 1:1 and these Figma-specific values may not
/// translate directly.
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_stack_child_properties;
///
/// let mut tree = json!({
///     "name": "Button",
///     "stackChildAlignSelf": "STRETCH",
///     "stackChildPrimaryGrow": 1.0,
///     "size": {"x": 100.0, "y": 48.0}
/// });
///
/// remove_stack_child_properties(&mut tree).unwrap();
///
/// assert!(tree.get("stackChildAlignSelf").is_none());
/// assert!(tree.get("stackChildPrimaryGrow").is_none());
/// assert!(tree.get("size").is_some());
/// ```
pub fn remove_stack_child_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove stack child properties
            map.remove("stackChildAlignSelf");
            map.remove("stackChildPrimaryGrow");

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
    fn test_removes_stack_child_align_self() {
        let mut tree = json!({
            "name": "Button",
            "stackChildAlignSelf": "STRETCH",
            "size": {"x": 100.0, "y": 48.0}
        });

        remove_stack_child_properties(&mut tree).unwrap();

        assert!(tree.get("stackChildAlignSelf").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_removes_stack_child_primary_grow() {
        let mut tree = json!({
            "name": "Text",
            "stackChildPrimaryGrow": 1.0,
            "fontSize": 14.0
        });

        remove_stack_child_properties(&mut tree).unwrap();

        assert!(tree.get("stackChildPrimaryGrow").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_removes_both_stack_child_properties() {
        let mut tree = json!({
            "name": "Row",
            "stackChildAlignSelf": "STRETCH",
            "stackChildPrimaryGrow": 1.0,
            "type": "FRAME"
        });

        remove_stack_child_properties(&mut tree).unwrap();

        assert!(tree.get("stackChildAlignSelf").is_none());
        assert!(tree.get("stackChildPrimaryGrow").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Row"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "children": [
                {
                    "name": "Child1",
                    "stackChildAlignSelf": "STRETCH"
                },
                {
                    "name": "Child2",
                    "stackChildPrimaryGrow": 1.0
                },
                {
                    "name": "Child3",
                    "stackChildAlignSelf": "CENTER",
                    "stackChildPrimaryGrow": 0.5
                }
            ]
        });

        remove_stack_child_properties(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("stackChildAlignSelf").is_none());
        assert!(children[1].get("stackChildPrimaryGrow").is_none());
        assert!(children[2].get("stackChildAlignSelf").is_none());
        assert!(children[2].get("stackChildPrimaryGrow").is_none());
        assert_eq!(children[0].get("name").unwrap().as_str(), Some("Child1"));
        assert_eq!(children[1].get("name").unwrap().as_str(), Some("Child2"));
        assert_eq!(children[2].get("name").unwrap().as_str(), Some("Child3"));
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Level1",
                    "stackChildPrimaryGrow": 1.0,
                    "children": [
                        {
                            "name": "Level2",
                            "stackChildAlignSelf": "STRETCH",
                            "stackChildPrimaryGrow": 2.0
                        }
                    ]
                }
            ]
        });

        remove_stack_child_properties(&mut tree).unwrap();

        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        assert!(level1.get("stackChildPrimaryGrow").is_none());
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("stackChildAlignSelf").is_none());
        assert!(level2.get("stackChildPrimaryGrow").is_none());
        assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
    }

    #[test]
    fn test_handles_missing_properties() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_stack_child_properties(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("type").is_some());
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_stack_child_properties(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "Button",
            "type": "FRAME",
            "stackChildAlignSelf": "STRETCH",
            "stackChildPrimaryGrow": 1.0,
            "stackMode": "HORIZONTAL",
            "size": {"x": 327.0, "y": 48.0},
            "cornerRadius": 12.0,
            "fillPaints": [{"color": "#343439", "type": "SOLID"}]
        });

        remove_stack_child_properties(&mut tree).unwrap();

        assert!(tree.get("stackChildAlignSelf").is_none());
        assert!(tree.get("stackChildPrimaryGrow").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("stackMode").unwrap().as_str(), Some("HORIZONTAL"));
        assert!(tree.get("size").is_some());
        assert_eq!(tree.get("cornerRadius").unwrap().as_f64(), Some(12.0));
        assert!(tree.get("fillPaints").is_some());
    }

    #[test]
    fn test_handles_multiple_occurrences_in_array() {
        let mut tree = json!({
            "children": [
                {"name": "A", "stackChildAlignSelf": "STRETCH"},
                {"name": "B", "stackChildPrimaryGrow": 1.0},
                {"name": "C", "stackChildAlignSelf": "CENTER", "stackChildPrimaryGrow": 0.5},
                {"name": "D"}
            ]
        });

        remove_stack_child_properties(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        for child in children {
            assert!(child.get("stackChildAlignSelf").is_none());
            assert!(child.get("stackChildPrimaryGrow").is_none());
            assert!(child.get("name").is_some());
        }
    }

    #[test]
    fn test_removes_from_real_world_example() {
        let mut tree = json!({
            "name": "Text",
            "size": {
                "x": 203.0,
                "y": 38.0
            },
            "stackChildPrimaryGrow": 1.0,
            "stackMode": "VERTICAL",
            "stackSpacing": 2.0,
            "children": [
                {
                    "name": "Members without roles",
                    "stackChildAlignSelf": "STRETCH",
                    "fontSize": 14.0
                },
                {
                    "name": "Default permissions",
                    "stackChildAlignSelf": "STRETCH",
                    "fontSize": 12.0
                }
            ]
        });

        remove_stack_child_properties(&mut tree).unwrap();

        assert!(tree.get("stackChildPrimaryGrow").is_none());
        assert!(tree.get("stackMode").is_some());

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("stackChildAlignSelf").is_none());
        assert!(children[1].get("stackChildAlignSelf").is_none());
        assert_eq!(children[0].get("fontSize").unwrap().as_f64(), Some(14.0));
        assert_eq!(children[1].get("fontSize").unwrap().as_f64(), Some(12.0));
    }
}
