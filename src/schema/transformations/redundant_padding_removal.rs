use crate::error::Result;
use serde_json::Value as JsonValue;

/// Removes redundant padding properties from the document tree.
///
/// This transformation removes duplicate padding fields when more general padding
/// properties already exist:
/// - Removes `stackPaddingRight` when `stackHorizontalPadding` exists
/// - Removes `stackPaddingBottom` when `stackVerticalPadding` exists
///
/// In Figma's auto-layout system, padding can be specified either with specific
/// side values (paddingRight, paddingBottom) or with axis-based values
/// (horizontalPadding, verticalPadding). When both exist, the specific values
/// are redundant and can be removed to reduce JSON size.
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_redundant_padding;
///
/// let mut tree = json!({
///     "name": "Button",
///     "stackHorizontalPadding": 20.0,
///     "stackPaddingRight": 20.0,  // redundant
///     "stackVerticalPadding": 14.0,
///     "stackPaddingBottom": 14.0  // redundant
/// });
///
/// remove_redundant_padding(&mut tree).unwrap();
///
/// assert!(tree.get("stackHorizontalPadding").is_some());
/// assert!(tree.get("stackPaddingRight").is_none());
/// assert!(tree.get("stackVerticalPadding").is_some());
/// assert!(tree.get("stackPaddingBottom").is_none());
/// ```
pub fn remove_redundant_padding(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove stackPaddingRight if stackHorizontalPadding exists
            if map.contains_key("stackHorizontalPadding") {
                map.remove("stackPaddingRight");
            }

            // Remove stackPaddingBottom if stackVerticalPadding exists
            if map.contains_key("stackVerticalPadding") {
                map.remove("stackPaddingBottom");
            }

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
    fn test_removes_padding_right_when_horizontal_exists() {
        let mut tree = json!({
            "name": "Button",
            "stackHorizontalPadding": 20.0,
            "stackPaddingRight": 20.0
        });

        remove_redundant_padding(&mut tree).unwrap();

        assert!(tree.get("stackHorizontalPadding").is_some());
        assert!(tree.get("stackPaddingRight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
    }

    #[test]
    fn test_removes_padding_bottom_when_vertical_exists() {
        let mut tree = json!({
            "name": "Button",
            "stackVerticalPadding": 14.0,
            "stackPaddingBottom": 14.0
        });

        remove_redundant_padding(&mut tree).unwrap();

        assert!(tree.get("stackVerticalPadding").is_some());
        assert!(tree.get("stackPaddingBottom").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
    }

    #[test]
    fn test_removes_both_redundant_paddings() {
        let mut tree = json!({
            "name": "Button",
            "stackHorizontalPadding": 20.0,
            "stackPaddingRight": 20.0,
            "stackVerticalPadding": 14.0,
            "stackPaddingBottom": 14.0
        });

        remove_redundant_padding(&mut tree).unwrap();

        assert!(tree.get("stackHorizontalPadding").is_some());
        assert!(tree.get("stackPaddingRight").is_none());
        assert!(tree.get("stackVerticalPadding").is_some());
        assert!(tree.get("stackPaddingBottom").is_none());
    }

    #[test]
    fn test_preserves_padding_right_when_no_horizontal() {
        let mut tree = json!({
            "name": "Button",
            "stackPaddingRight": 20.0,
            "stackPaddingBottom": 14.0
        });

        remove_redundant_padding(&mut tree).unwrap();

        assert!(tree.get("stackPaddingRight").is_some());
        assert!(tree.get("stackPaddingBottom").is_some());
    }

    #[test]
    fn test_preserves_padding_bottom_when_no_vertical() {
        let mut tree = json!({
            "name": "Button",
            "stackHorizontalPadding": 20.0,
            "stackPaddingBottom": 14.0
        });

        remove_redundant_padding(&mut tree).unwrap();

        assert!(tree.get("stackHorizontalPadding").is_some());
        assert!(tree.get("stackPaddingBottom").is_some());
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "children": [
                {
                    "name": "Child1",
                    "stackHorizontalPadding": 20.0,
                    "stackPaddingRight": 20.0
                },
                {
                    "name": "Child2",
                    "stackVerticalPadding": 14.0,
                    "stackPaddingBottom": 14.0
                }
            ]
        });

        remove_redundant_padding(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("stackHorizontalPadding").is_some());
        assert!(children[0].get("stackPaddingRight").is_none());
        assert!(children[1].get("stackVerticalPadding").is_some());
        assert!(children[1].get("stackPaddingBottom").is_none());
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "stackHorizontalPadding": 16.0,
            "stackPaddingRight": 16.0,
            "children": [
                {
                    "name": "Level1",
                    "children": [
                        {
                            "name": "Level2",
                            "stackVerticalPadding": 12.0,
                            "stackPaddingBottom": 12.0
                        }
                    ]
                }
            ]
        });

        remove_redundant_padding(&mut tree).unwrap();

        assert!(tree.get("stackHorizontalPadding").is_some());
        assert!(tree.get("stackPaddingRight").is_none());

        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("stackVerticalPadding").is_some());
        assert!(level2.get("stackPaddingBottom").is_none());
    }

    #[test]
    fn test_handles_missing_properties() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_redundant_padding(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("type").is_some());
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_redundant_padding(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "Button",
            "cornerRadius": 12.0,
            "stackHorizontalPadding": 20.0,
            "stackPaddingRight": 20.0,
            "stackVerticalPadding": 14.0,
            "stackPaddingBottom": 14.0,
            "stackMode": "HORIZONTAL",
            "fillPaints": [{"color": "#1461f6", "type": "SOLID"}]
        });

        remove_redundant_padding(&mut tree).unwrap();

        assert!(tree.get("stackPaddingRight").is_none());
        assert!(tree.get("stackPaddingBottom").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
        assert_eq!(tree.get("cornerRadius").unwrap().as_f64(), Some(12.0));
        assert_eq!(tree.get("stackMode").unwrap().as_str(), Some("HORIZONTAL"));
        assert!(tree.get("fillPaints").is_some());
    }

    #[test]
    fn test_handles_different_padding_values() {
        // Even if values differ, remove the redundant one
        let mut tree = json!({
            "name": "Button",
            "stackHorizontalPadding": 20.0,
            "stackPaddingRight": 25.0  // different value but still redundant
        });

        remove_redundant_padding(&mut tree).unwrap();

        assert!(tree.get("stackHorizontalPadding").is_some());
        assert!(tree.get("stackPaddingRight").is_none());
    }

    #[test]
    fn test_real_world_example_from_roles_members() {
        let mut tree = json!({
            "name": "Button",
            "cornerRadius": 12.0,
            "fillPaints": [
                {
                    "color": "#343439",
                    "type": "SOLID"
                }
            ],
            "stackCounterAlignItems": "CENTER",
            "stackHorizontalPadding": 20.0,
            "stackMode": "HORIZONTAL",
            "stackPaddingBottom": 14.0,
            "stackPaddingRight": 20.0,
            "stackVerticalPadding": 14.0,
            "type": "INSTANCE"
        });

        remove_redundant_padding(&mut tree).unwrap();

        assert!(tree.get("stackHorizontalPadding").is_some());
        assert_eq!(tree.get("stackHorizontalPadding").unwrap().as_f64(), Some(20.0));
        assert!(tree.get("stackPaddingRight").is_none());

        assert!(tree.get("stackVerticalPadding").is_some());
        assert_eq!(tree.get("stackVerticalPadding").unwrap().as_f64(), Some(14.0));
        assert!(tree.get("stackPaddingBottom").is_none());

        // Other fields preserved
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
        assert!(tree.get("cornerRadius").is_some());
        assert!(tree.get("stackMode").is_some());
    }
}
