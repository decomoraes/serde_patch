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

/// Computes a JSON diff suitable for use as a Merge Patch (RFC 7396).
///
/// Returns a `serde_json::Value` containing only changed fields (with new values).
/// If no changes, returns an empty object.
///
/// See also [`diff_including`] for a version that can force inclusion of specific fields.
///
/// # Example
///
/// ```
/// use serde_json::json;
///
/// #[derive(serde::Serialize)]
/// struct User { id: u32, name: String, age: u8 }
///
/// let old = User { id: 1, name: "old".to_string(), age: 31 };
/// let new = User { id: 1, name: "new".to_string(), age: 31 };
///
/// let patch = serde_patch::diff(&old, &new).unwrap();
/// assert_eq!(patch, json!({ "name": "new" }));
/// ```
pub fn diff<T: serde::Serialize>(old: &T, new: &T) -> Result<serde_json::Value, serde_json::Error> {
    let old_val = serde_json::to_value(old)?;
    let new_val = serde_json::to_value(new)?;
    let diff_opt = compute_diff(Some(&old_val), &new_val, &HashSet::new(), "");
    Ok(diff_opt.unwrap_or(serde_json::Value::Object(serde_json::Map::new())))
}

/// Computes a JSON diff, forcing specific fields to be included even if unchanged.
///
/// This is useful when you need to provide context (like an ID) in the patch,
/// regardless of whether that field has changed.
///
/// # Example
///
/// ```
/// use serde_json::json;
///
/// #[derive(serde::Serialize)]
/// struct User { id: u32, name: String }
///
/// let old = User { id: 1, name: "old".to_string() };
/// let new = User { id: 1, name: "new".to_string() };
///
/// // "id" is included even though it didn't change
/// let patch = serde_patch::diff_including(&old, &new, &["id"]).unwrap();
/// assert_eq!(patch, json!({ "id": 1, "name": "new" }));
/// ```
pub fn diff_including<T: serde::Serialize>(
    old: &T,
    new: &T,
    including: &[&str],
) -> Result<serde_json::Value, serde_json::Error> {
    let old_val = serde_json::to_value(old)?;
    let new_val = serde_json::to_value(new)?;
    let including_set: HashSet<String> = including.iter().map(|s| s.to_string()).collect();
    let diff_opt = compute_diff(Some(&old_val), &new_val, &including_set, "");
    Ok(diff_opt.unwrap_or(serde_json::Value::Object(serde_json::Map::new())))
}
