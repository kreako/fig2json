use crate::error::Result;
use serde_json::Value as JsonValue;

/// Removes Figma-specific stack alignment properties from the document tree.
///
/// This transformation removes the following fields:
/// - `stackCounterAlignItems`: Controls alignment of items along the cross axis (perpendicular to stack direction)
/// - `stackPrimaryAlignItems`: Controls alignment/distribution of items along the main axis (parallel to stack direction)
///
/// These properties are specific to Figma's auto-layout configuration and are not directly
/// needed for HTML/CSS rendering. CSS uses different mechanisms (flexbox `align-items`, `justify-content`, etc.)
/// for similar behavior, but the Figma-specific values don't translate 1:1.
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_stack_align_items;
///
/// let mut tree = json!({
///     "name": "Row",
///     "stackMode": "HORIZONTAL",
///     "stackCounterAlignItems": "CENTER",
///     "stackPrimaryAlignItems": "SPACE_BETWEEN",
///     "size": {"x": 327.0, "y": 40.0}
/// });
///
/// remove_stack_align_items(&mut tree).unwrap();
///
/// assert!(tree.get("stackCounterAlignItems").is_none());
/// assert!(tree.get("stackPrimaryAlignItems").is_none());
/// assert!(tree.get("stackMode").is_some());
/// ```
pub fn remove_stack_align_items(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove stack alignment properties
            map.remove("stackCounterAlignItems");
            map.remove("stackPrimaryAlignItems");

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
    fn test_removes_stack_counter_align_items() {
        let mut tree = json!({
            "name": "Row",
            "stackMode": "HORIZONTAL",
            "stackCounterAlignItems": "CENTER",
            "size": {"x": 327.0, "y": 40.0}
        });

        remove_stack_align_items(&mut tree).unwrap();

        assert!(tree.get("stackCounterAlignItems").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Row"));
        assert_eq!(tree.get("stackMode").unwrap().as_str(), Some("HORIZONTAL"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_removes_stack_primary_align_items() {
        let mut tree = json!({
            "name": "Header",
            "stackMode": "HORIZONTAL",
            "stackPrimaryAlignItems": "SPACE_BETWEEN",
            "stackSpacing": 116.0
        });

        remove_stack_align_items(&mut tree).unwrap();

        assert!(tree.get("stackPrimaryAlignItems").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Header"));
        assert_eq!(tree.get("stackSpacing").unwrap().as_f64(), Some(116.0));
    }

    #[test]
    fn test_removes_both_stack_align_properties() {
        let mut tree = json!({
            "name": "Container",
            "stackMode": "VERTICAL",
            "stackCounterAlignItems": "CENTER",
            "stackPrimaryAlignItems": "SPACE_EVENLY",
            "type": "FRAME"
        });

        remove_stack_align_items(&mut tree).unwrap();

        assert!(tree.get("stackCounterAlignItems").is_none());
        assert!(tree.get("stackPrimaryAlignItems").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Container"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "children": [
                {
                    "name": "Child1",
                    "stackCounterAlignItems": "CENTER"
                },
                {
                    "name": "Child2",
                    "stackPrimaryAlignItems": "SPACE_BETWEEN"
                },
                {
                    "name": "Child3",
                    "stackCounterAlignItems": "STRETCH",
                    "stackPrimaryAlignItems": "CENTER"
                }
            ]
        });

        remove_stack_align_items(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("stackCounterAlignItems").is_none());
        assert!(children[1].get("stackPrimaryAlignItems").is_none());
        assert!(children[2].get("stackCounterAlignItems").is_none());
        assert!(children[2].get("stackPrimaryAlignItems").is_none());
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
                    "stackPrimaryAlignItems": "SPACE_BETWEEN",
                    "children": [
                        {
                            "name": "Level2",
                            "stackCounterAlignItems": "CENTER",
                            "stackPrimaryAlignItems": "SPACE_EVENLY"
                        }
                    ]
                }
            ]
        });

        remove_stack_align_items(&mut tree).unwrap();

        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        assert!(level1.get("stackPrimaryAlignItems").is_none());
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("stackCounterAlignItems").is_none());
        assert!(level2.get("stackPrimaryAlignItems").is_none());
        assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
    }

    #[test]
    fn test_handles_missing_properties() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_stack_align_items(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("type").is_some());
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_stack_align_items(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_stack_properties() {
        let mut tree = json!({
            "name": "Button",
            "type": "FRAME",
            "stackCounterAlignItems": "CENTER",
            "stackPrimaryAlignItems": "CENTER",
            "stackMode": "HORIZONTAL",
            "stackSpacing": 4.0,
            "stackHorizontalPadding": 20.0,
            "stackVerticalPadding": 14.0,
            "size": {"x": 327.0, "y": 48.0},
            "cornerRadius": 12.0
        });

        remove_stack_align_items(&mut tree).unwrap();

        assert!(tree.get("stackCounterAlignItems").is_none());
        assert!(tree.get("stackPrimaryAlignItems").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
        assert_eq!(tree.get("stackMode").unwrap().as_str(), Some("HORIZONTAL"));
        assert_eq!(tree.get("stackSpacing").unwrap().as_f64(), Some(4.0));
        assert_eq!(tree.get("stackHorizontalPadding").unwrap().as_f64(), Some(20.0));
        assert_eq!(tree.get("stackVerticalPadding").unwrap().as_f64(), Some(14.0));
        assert!(tree.get("size").is_some());
        assert_eq!(tree.get("cornerRadius").unwrap().as_f64(), Some(12.0));
    }

    #[test]
    fn test_handles_multiple_occurrences_in_array() {
        let mut tree = json!({
            "children": [
                {"name": "A", "stackCounterAlignItems": "CENTER"},
                {"name": "B", "stackPrimaryAlignItems": "SPACE_BETWEEN"},
                {"name": "C", "stackCounterAlignItems": "STRETCH", "stackPrimaryAlignItems": "SPACE_EVENLY"},
                {"name": "D"}
            ]
        });

        remove_stack_align_items(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        for child in children {
            assert!(child.get("stackCounterAlignItems").is_none());
            assert!(child.get("stackPrimaryAlignItems").is_none());
            assert!(child.get("name").is_some());
        }
    }

    #[test]
    fn test_removes_from_real_world_example() {
        let mut tree = json!({
            "name": "header",
            "size": {
                "x": 327.0,
                "y": 40.0
            },
            "stackCounterAlignItems": "CENTER",
            "stackMode": "HORIZONTAL",
            "stackPrimaryAlignItems": "SPACE_BETWEEN",
            "stackSpacing": 116.0,
            "transform": {
                "x": 24.0,
                "y": 136.0
            },
            "children": [
                {
                    "name": "title",
                    "fontSize": 18.0
                },
                {
                    "name": "icon",
                    "size": {"x": 20.0, "y": 20.0}
                }
            ]
        });

        remove_stack_align_items(&mut tree).unwrap();

        assert!(tree.get("stackCounterAlignItems").is_none());
        assert!(tree.get("stackPrimaryAlignItems").is_none());
        assert!(tree.get("stackMode").is_some());
        assert!(tree.get("stackSpacing").is_some());
        assert!(tree.get("transform").is_some());

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert_eq!(children[0].get("fontSize").unwrap().as_f64(), Some(18.0));
        assert!(children[1].get("size").is_some());
    }
}
