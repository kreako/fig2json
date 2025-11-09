use crate::error::Result;
use serde_json::Value as JsonValue;

/// Removes Figma-specific stack sizing properties from the document tree.
///
/// This transformation removes the following fields:
/// - `stackCounterSizing`: Controls how nodes resize along the counter-axis in Figma's auto-layout
/// - `stackPrimarySizing`: Controls how nodes resize along the primary axis in Figma's auto-layout
///
/// These properties are specific to Figma's auto-layout sizing system and are not directly
/// needed for HTML/CSS rendering, as CSS uses different mechanisms (flexbox, grid, etc.)
/// for sizing behavior.
///
/// Common values include:
/// - `RESIZE_TO_FIT_WITH_IMPLICIT_SIZE`
/// - `FIXED`
/// - `AUTO`
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_stack_sizing_properties;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "stackCounterSizing": "RESIZE_TO_FIT_WITH_IMPLICIT_SIZE",
///     "stackPrimarySizing": "FIXED",
///     "size": {"x": 100.0, "y": 100.0}
/// });
///
/// remove_stack_sizing_properties(&mut tree).unwrap();
///
/// assert!(tree.get("stackCounterSizing").is_none());
/// assert!(tree.get("stackPrimarySizing").is_none());
/// assert!(tree.get("size").is_some());
/// ```
pub fn remove_stack_sizing_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove stack sizing properties
            map.remove("stackCounterSizing");
            map.remove("stackPrimarySizing");

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
    fn test_removes_stack_counter_sizing() {
        let mut tree = json!({
            "name": "Frame",
            "stackCounterSizing": "RESIZE_TO_FIT_WITH_IMPLICIT_SIZE",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_stack_sizing_properties(&mut tree).unwrap();

        assert!(tree.get("stackCounterSizing").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_removes_stack_primary_sizing() {
        let mut tree = json!({
            "name": "Frame",
            "stackPrimarySizing": "FIXED",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_stack_sizing_properties(&mut tree).unwrap();

        assert!(tree.get("stackPrimarySizing").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_removes_both_sizing_properties() {
        let mut tree = json!({
            "name": "Frame",
            "stackCounterSizing": "RESIZE_TO_FIT_WITH_IMPLICIT_SIZE",
            "stackPrimarySizing": "FIXED",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_stack_sizing_properties(&mut tree).unwrap();

        assert!(tree.get("stackCounterSizing").is_none());
        assert!(tree.get("stackPrimarySizing").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "children": [
                {
                    "name": "Child1",
                    "stackCounterSizing": "AUTO",
                    "stackPrimarySizing": "FIXED"
                },
                {
                    "name": "Child2",
                    "stackPrimarySizing": "RESIZE_TO_FIT_WITH_IMPLICIT_SIZE"
                }
            ]
        });

        remove_stack_sizing_properties(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("stackCounterSizing").is_none());
        assert!(children[0].get("stackPrimarySizing").is_none());
        assert!(children[1].get("stackPrimarySizing").is_none());
        assert_eq!(children[0].get("name").unwrap().as_str(), Some("Child1"));
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "stackCounterSizing": "FIXED",
            "children": [
                {
                    "name": "Level1",
                    "stackPrimarySizing": "AUTO",
                    "children": [
                        {
                            "name": "Level2",
                            "stackCounterSizing": "RESIZE_TO_FIT_WITH_IMPLICIT_SIZE",
                            "stackPrimarySizing": "FIXED"
                        }
                    ]
                }
            ]
        });

        remove_stack_sizing_properties(&mut tree).unwrap();

        assert!(tree.get("stackCounterSizing").is_none());
        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        assert!(level1.get("stackPrimarySizing").is_none());
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("stackCounterSizing").is_none());
        assert!(level2.get("stackPrimarySizing").is_none());
        assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
    }

    #[test]
    fn test_handles_missing_sizing_properties() {
        let mut tree = json!({
            "name": "Frame",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_stack_sizing_properties(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_stack_sizing_properties(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "stackCounterSizing": "AUTO",
            "stackPrimarySizing": "FIXED",
            "stackMode": "HORIZONTAL",
            "stackSpacing": 16.0,
            "size": {"x": 100.0, "y": 100.0},
            "transform": {"x": 10.0, "y": 20.0}
        });

        remove_stack_sizing_properties(&mut tree).unwrap();

        assert!(tree.get("stackCounterSizing").is_none());
        assert!(tree.get("stackPrimarySizing").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("stackMode").unwrap().as_str(), Some("HORIZONTAL"));
        assert_eq!(tree.get("stackSpacing").unwrap().as_f64(), Some(16.0));
        assert!(tree.get("size").is_some());
        assert!(tree.get("transform").is_some());
    }

    #[test]
    fn test_handles_multiple_occurrences_in_array() {
        let mut tree = json!({
            "children": [
                {"name": "A", "stackCounterSizing": "FIXED"},
                {"name": "B", "stackPrimarySizing": "AUTO"},
                {"name": "C", "stackCounterSizing": "RESIZE_TO_FIT_WITH_IMPLICIT_SIZE", "stackPrimarySizing": "FIXED"},
                {"name": "D"}
            ]
        });

        remove_stack_sizing_properties(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        for child in children {
            assert!(child.get("stackCounterSizing").is_none());
            assert!(child.get("stackPrimarySizing").is_none());
            assert!(child.get("name").is_some());
        }
    }
}
