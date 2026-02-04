use serde_json::{Map, Value};
use std::collections::HashSet;

/// Recursively computes a JSON diff between two values (internal).
///
/// Returns a partial JSON value containing only changed fields (new values)
/// and optionally forced fields (even if unchanged).
///
/// If no differences (and no forced fields apply), returns `Value::Null` or an empty object.
pub fn compute_diff(
    old: Option<&Value>,
    new: &Value,
    forced: &HashSet<String>,
    current_path: &str,
) -> Option<Value> {
    if let (Some(old_obj), Value::Object(new_map)) = (old.and_then(|v| v.as_object()), new) {
        let old_map = old_obj;
        let mut diff_map: Map<String, Value> = Map::new();

        for (key, new_value) in new_map {
            let full_path = if current_path.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", current_path, key)
            };

            let old_value = old_map.get(key);

            if let Some(diff_value) = compute_diff(old_value, new_value, forced, &full_path) {
                diff_map.insert(key.clone(), diff_value);
            } else if forced.contains(&full_path) {
                diff_map.insert(key.clone(), new_value.clone());
            }
        }

        for key in old_map.keys() {
            if !new_map.contains_key(key) {
                diff_map.insert(key.clone(), Value::Null);
            }
        }

        if diff_map.is_empty() {
            None
        } else {
            Some(Value::Object(diff_map))
        }
    } else {
        let equal = old == Some(new);
        if equal && !forced.contains(current_path) {
            None
        } else {
            Some(new.clone())
        }
    }
}

/// Returns true if a diff value represents no changes (internal).
pub fn is_empty_diff(value: &Value) -> bool {
    matches!(value, Value::Object(map) if map.is_empty()) || value.is_null()
}

/// Computes a JSON diff suitable for use as a Merge Patch (RFC 7396).
///
/// Returns a `serde_json::Value` containing only changed fields (with new values).
/// If no changes, returns an empty object.
///
/// The variant with forced fields includes specified paths even if unchanged.
///
/// # Example
///
/// ```
/// use serde_patch::diff;
/// use serde_json::json;
///
/// #[derive(serde::Serialize)]
/// struct User { id: u32, name: String, age: u8 }
///
/// let old = User { id: 1, name: "old".to_string(), age: 31 };
/// let new = User { id: 1, name: "new".to_string(), age: 31 };
///
/// let patch = diff!(old, new).unwrap();
/// assert_eq!(patch, json!({ "name": "new" }));
///
/// // Force inclusion of "id" even though unchanged
/// let patch_forced = diff!(old, new; ["id"]).unwrap();
/// assert_eq!(patch_forced, json!({ "id": 1, "name": "new" }));
/// ```
#[macro_export]
macro_rules! diff {
    ($old:expr, $new:expr) => {{
        (|| -> Result<serde_json::Value, serde_json::Error> {
            use serde_json::Value;
            use std::collections::HashSet;

            let old_val = serde_json::to_value(&$old)?;
            let new_val = serde_json::to_value(&$new)?;
            let diff_opt = $crate::diff_patch::compute_diff(Some(&old_val), &new_val, &HashSet::new(), "");
            Ok(diff_opt.unwrap_or(Value::Object(serde_json::Map::new())))
        })()
    }};

    ($old:expr, $new:expr; [ $($field:expr),* ]) => {{
        (|| -> Result<serde_json::Value, serde_json::Error> {
            use serde_json::{Map, Value};
            use std::collections::HashSet;

            let forced: HashSet<String> = [ $( $field.to_string() ),* ].into_iter().collect();
            let old_val = serde_json::to_value(&$old)?;
            let new_val = serde_json::to_value(&$new)?;
            let diff_opt = $crate::diff_patch::compute_diff(Some(&old_val), &new_val, &forced, "");
            Ok(diff_opt.unwrap_or(Value::Object(Map::new())))
        })()
    }};
}
