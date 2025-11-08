use crate::error::Result;
use serde_json::Value as JsonValue;

/// Remove userFacingVersion fields from all objects in the JSON tree
///
/// Recursively traverses the JSON tree and removes all "userFacingVersion" fields.
/// These fields contain Figma version strings that are not needed for HTML/CSS rendering.
///
/// # Arguments
/// * `tree` - The JSON tree to modify (usually the document root)
///
/// # Returns
/// * `Ok(())` - Successfully removed all userFacingVersion fields
///
/// # Examples
/// ```no_run
/// use fig2json::schema::remove_user_facing_versions;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Component",
///     "type": "INSTANCE",
///     "userFacingVersion": "434:917",
///     "visible": true
/// });
/// remove_user_facing_versions(&mut tree).unwrap();
/// // tree now has only "name", "type", and "visible" fields
/// ```
pub fn remove_user_facing_versions(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// Recursively remove userFacingVersion fields from a JSON value
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // Remove the "userFacingVersion" field if it exists
            map.remove("userFacingVersion");

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
    fn test_remove_user_facing_version_simple() {
        let mut tree = json!({
            "name": "Component",
            "type": "INSTANCE",
            "userFacingVersion": "434:917",
            "visible": true
        });

        remove_user_facing_versions(&mut tree).unwrap();

        assert!(tree.get("userFacingVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Component"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("INSTANCE"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_user_facing_version_different_formats() {
        let mut tree = json!({
            "items": [
                {"userFacingVersion": "1:10", "name": "Item1"},
                {"userFacingVersion": "6:99485", "name": "Item2"},
                {"userFacingVersion": "8185:4", "name": "Item3"}
            ]
        });

        remove_user_facing_versions(&mut tree).unwrap();

        assert!(tree["items"][0].get("userFacingVersion").is_none());
        assert!(tree["items"][1].get("userFacingVersion").is_none());
        assert!(tree["items"][2].get("userFacingVersion").is_none());
        assert_eq!(tree["items"][0]["name"].as_str(), Some("Item1"));
        assert_eq!(tree["items"][1]["name"].as_str(), Some("Item2"));
        assert_eq!(tree["items"][2]["name"].as_str(), Some("Item3"));
    }

    #[test]
    fn test_remove_user_facing_version_nested() {
        let mut tree = json!({
            "name": "Root",
            "userFacingVersion": "1:1",
            "children": [
                {
                    "name": "Child1",
                    "userFacingVersion": "2:5"
                },
                {
                    "name": "Child2",
                    "userFacingVersion": "3:10"
                }
            ]
        });

        remove_user_facing_versions(&mut tree).unwrap();

        assert!(tree.get("userFacingVersion").is_none());
        assert!(tree["children"][0].get("userFacingVersion").is_none());
        assert!(tree["children"][1].get("userFacingVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Root"));
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_remove_user_facing_version_deeply_nested() {
        let mut tree = json!({
            "document": {
                "userFacingVersion": "1:1",
                "children": [
                    {
                        "userFacingVersion": "2:1",
                        "children": [
                            {
                                "userFacingVersion": "3:1",
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_user_facing_versions(&mut tree).unwrap();

        assert!(tree["document"].get("userFacingVersion").is_none());
        assert!(tree["document"]["children"][0]
            .get("userFacingVersion")
            .is_none());
        assert!(tree["document"]["children"][0]["children"][0]
            .get("userFacingVersion")
            .is_none());
        assert_eq!(
            tree["document"]["children"][0]["children"][0]
                .get("name")
                .unwrap()
                .as_str(),
            Some("DeepChild")
        );
    }

    #[test]
    fn test_remove_user_facing_version_missing() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "visible": true
        });

        remove_user_facing_versions(&mut tree).unwrap();

        assert!(tree.get("userFacingVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_user_facing_version_preserves_other_fields() {
        let mut tree = json!({
            "name": "Instance",
            "type": "INSTANCE",
            "userFacingVersion": "1000:5000",
            "size": {"x": 100, "y": 200},
            "opacity": 0.8,
            "visible": true
        });

        remove_user_facing_versions(&mut tree).unwrap();

        assert!(tree.get("userFacingVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Instance"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("INSTANCE"));
        assert_eq!(tree["size"]["x"].as_i64(), Some(100));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.8));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_user_facing_version_in_arrays() {
        let mut tree = json!({
            "instances": [
                {
                    "userFacingVersion": "1:1",
                    "name": "Instance1"
                },
                {
                    "userFacingVersion": "2:2",
                    "name": "Instance2"
                }
            ]
        });

        remove_user_facing_versions(&mut tree).unwrap();

        assert!(tree["instances"][0].get("userFacingVersion").is_none());
        assert_eq!(
            tree["instances"][0].get("name").unwrap().as_str(),
            Some("Instance1")
        );
        assert!(tree["instances"][1].get("userFacingVersion").is_none());
        assert_eq!(
            tree["instances"][1].get("name").unwrap().as_str(),
            Some("Instance2")
        );
    }

    #[test]
    fn test_remove_user_facing_version_empty_object() {
        let mut tree = json!({});

        remove_user_facing_versions(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_remove_user_facing_version_primitives() {
        let mut tree = json!("test");

        remove_user_facing_versions(&mut tree).unwrap();

        assert_eq!(tree.as_str(), Some("test"));
    }

    #[test]
    fn test_remove_user_facing_version_mixed_nodes() {
        let mut tree = json!({
            "children": [
                {
                    "name": "WithVersion",
                    "userFacingVersion": "1:1"
                },
                {
                    "name": "WithoutVersion"
                },
                {
                    "name": "AlsoWithVersion",
                    "userFacingVersion": "2:2"
                }
            ]
        });

        remove_user_facing_versions(&mut tree).unwrap();

        assert!(tree["children"][0].get("userFacingVersion").is_none());
        assert!(tree["children"][1].get("userFacingVersion").is_none());
        assert!(tree["children"][2].get("userFacingVersion").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("WithVersion"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("WithoutVersion"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("AlsoWithVersion"));
    }
}
