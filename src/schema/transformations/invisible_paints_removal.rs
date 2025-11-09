use crate::error::Result;
use serde_json::Value as JsonValue;

/// Removes invisible paints from fillPaints and strokePaints arrays.
///
/// This transformation filters out paint objects from `fillPaints` and `strokePaints`
/// arrays where the `visible` property is explicitly set to `false`. Invisible paints
/// do not render in the final output and are not needed for HTML/CSS conversion.
///
/// Paint objects without a `visible` property are assumed to be visible and are preserved.
/// Paint objects with `visible: true` are also preserved (though the `visible` property
/// itself may be removed by other transformations that handle default values).
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_invisible_paints;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "fillPaints": [
///         {
///             "color": "#ffffff",
///             "type": "SOLID",
///             "visible": false
///         },
///         {
///             "color": "#000000",
///             "type": "SOLID"
///         }
///     ]
/// });
///
/// remove_invisible_paints(&mut tree).unwrap();
///
/// let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
/// assert_eq!(fills.len(), 1);
/// assert_eq!(fills[0].get("color").unwrap().as_str(), Some("#000000"));
/// ```
pub fn remove_invisible_paints(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Filter fillPaints array
            if let Some(JsonValue::Array(paints)) = map.get_mut("fillPaints") {
                paints.retain(|paint| !is_invisible(paint));
            }

            // Filter strokePaints array
            if let Some(JsonValue::Array(paints)) = map.get_mut("strokePaints") {
                paints.retain(|paint| !is_invisible(paint));
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

/// Checks if a paint object is invisible (has visible: false)
fn is_invisible(paint: &JsonValue) -> bool {
    if let Some(visible) = paint.get("visible") {
        if let Some(visible_bool) = visible.as_bool() {
            return !visible_bool;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_removes_invisible_fill_paint() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillPaints": [
                {
                    "color": "#ffffff",
                    "type": "SOLID",
                    "visible": false
                }
            ]
        });

        remove_invisible_paints(&mut tree).unwrap();

        let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
        assert_eq!(fills.len(), 0);
    }

    #[test]
    fn test_preserves_visible_fill_paint() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillPaints": [
                {
                    "color": "#000000",
                    "type": "SOLID",
                    "visible": true
                }
            ]
        });

        remove_invisible_paints(&mut tree).unwrap();

        let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].get("color").unwrap().as_str(), Some("#000000"));
    }

    #[test]
    fn test_preserves_paint_without_visible_property() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillPaints": [
                {
                    "color": "#ff0000",
                    "type": "SOLID"
                }
            ]
        });

        remove_invisible_paints(&mut tree).unwrap();

        let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].get("color").unwrap().as_str(), Some("#ff0000"));
    }

    #[test]
    fn test_filters_mixed_visible_invisible_fills() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillPaints": [
                {
                    "color": "#ffffff",
                    "type": "SOLID",
                    "visible": false
                },
                {
                    "color": "#000000",
                    "type": "SOLID"
                },
                {
                    "color": "#ff0000",
                    "type": "SOLID",
                    "visible": true
                }
            ]
        });

        remove_invisible_paints(&mut tree).unwrap();

        let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
        assert_eq!(fills.len(), 2);
        assert_eq!(fills[0].get("color").unwrap().as_str(), Some("#000000"));
        assert_eq!(fills[1].get("color").unwrap().as_str(), Some("#ff0000"));
    }

    #[test]
    fn test_removes_invisible_stroke_paint() {
        let mut tree = json!({
            "name": "Rectangle",
            "strokePaints": [
                {
                    "color": "#343439",
                    "type": "SOLID",
                    "visible": false
                }
            ]
        });

        remove_invisible_paints(&mut tree).unwrap();

        let strokes = tree.get("strokePaints").unwrap().as_array().unwrap();
        assert_eq!(strokes.len(), 0);
    }

    #[test]
    fn test_filters_both_fill_and_stroke_paints() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillPaints": [
                {
                    "color": "#ffffff",
                    "type": "SOLID",
                    "visible": false
                },
                {
                    "color": "#000000",
                    "type": "SOLID"
                }
            ],
            "strokePaints": [
                {
                    "color": "#ff0000",
                    "type": "SOLID",
                    "visible": false
                },
                {
                    "color": "#00ff00",
                    "type": "SOLID",
                    "visible": true
                }
            ]
        });

        remove_invisible_paints(&mut tree).unwrap();

        let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].get("color").unwrap().as_str(), Some("#000000"));

        let strokes = tree.get("strokePaints").unwrap().as_array().unwrap();
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].get("color").unwrap().as_str(), Some("#00ff00"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "children": [
                {
                    "name": "Child1",
                    "fillPaints": [
                        {"color": "#ffffff", "type": "SOLID", "visible": false},
                        {"color": "#000000", "type": "SOLID"}
                    ]
                },
                {
                    "name": "Child2",
                    "strokePaints": [
                        {"color": "#ff0000", "type": "SOLID", "visible": false}
                    ]
                }
            ]
        });

        remove_invisible_paints(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        let child1_fills = children[0].get("fillPaints").unwrap().as_array().unwrap();
        assert_eq!(child1_fills.len(), 1);
        assert_eq!(child1_fills[0].get("color").unwrap().as_str(), Some("#000000"));

        let child2_strokes = children[1].get("strokePaints").unwrap().as_array().unwrap();
        assert_eq!(child2_strokes.len(), 0);
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "fillPaints": [
                {"color": "#aaa", "type": "SOLID", "visible": false}
            ],
            "children": [
                {
                    "name": "Level1",
                    "children": [
                        {
                            "name": "Level2",
                            "fillPaints": [
                                {"color": "#fff", "type": "SOLID", "visible": false},
                                {"color": "#000", "type": "SOLID"}
                            ]
                        }
                    ]
                }
            ]
        });

        remove_invisible_paints(&mut tree).unwrap();

        let root_fills = tree.get("fillPaints").unwrap().as_array().unwrap();
        assert_eq!(root_fills.len(), 0);

        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        let level2_fills = level2.get("fillPaints").unwrap().as_array().unwrap();
        assert_eq!(level2_fills.len(), 1);
        assert_eq!(level2_fills[0].get("color").unwrap().as_str(), Some("#000"));
    }

    #[test]
    fn test_handles_missing_paint_arrays() {
        let mut tree = json!({
            "name": "Rectangle",
            "type": "FRAME"
        });

        remove_invisible_paints(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_handles_empty_paint_arrays() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillPaints": [],
            "strokePaints": []
        });

        remove_invisible_paints(&mut tree).unwrap();

        let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
        let strokes = tree.get("strokePaints").unwrap().as_array().unwrap();
        assert_eq!(fills.len(), 0);
        assert_eq!(strokes.len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "icon/ai",
            "type": "INSTANCE",
            "fillPaints": [
                {
                    "color": "#ffffff",
                    "type": "SOLID",
                    "visible": false
                }
            ],
            "size": {"x": 20.0, "y": 20.0},
            "transform": {"x": 0.0, "y": 9.0}
        });

        remove_invisible_paints(&mut tree).unwrap();

        let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
        assert_eq!(fills.len(), 0);
        assert_eq!(tree.get("name").unwrap().as_str(), Some("icon/ai"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("INSTANCE"));
        assert!(tree.get("size").is_some());
        assert!(tree.get("transform").is_some());
    }
}
