use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

/// Applies a JSON Merge Patch (RFC 7396) in-place.
///
/// Modifies the current value directly.
///
/// # Example
///
/// ```rust
/// #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
/// struct User { id: u32, name: String }
///
/// let mut user = User { id: 1, name: "old".to_string() };
/// let patch = r#"{ "name": "new" }"#;
///
/// serde_patch::apply_mut(&mut user, patch).unwrap();
/// assert_eq!(user.name, "new");
/// assert_eq!(user.id, 1);
/// ```
pub fn apply_mut<T, P>(current: &mut T, patch: P) -> Result<(), serde_json::Error>
where
    T: Serialize + DeserializeOwned,
    P: AsRef<[u8]>,
{
    let mut current_val = serde_json::to_value(&current)?;
    let patch_val: Value = serde_json::from_slice(patch.as_ref())?;
    merge_patch(&mut current_val, &patch_val);
    *current = serde_json::from_value(current_val)?;
    Ok(())
}

/// Recursively merges a patch into a target JSON value (internal).
fn merge_patch(target: &mut Value, patch: &Value) {
    if let Value::Object(patch_map) = patch {
        if !target.is_object() {
            *target = Value::Object(Map::new());
        }
        let target_map = target.as_object_mut().unwrap();
        for (key, patch_value) in patch_map {
            if patch_value.is_null() {
                target_map.remove(key);
            } else {
                let target_entry = target_map.entry(key.clone()).or_insert(Value::Null);
                merge_patch(target_entry, patch_value);
            }
        }
    } else {
        *target = patch.clone();
    }
}
