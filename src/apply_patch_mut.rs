use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

/// Applies a JSON Merge Patch (RFC 7396) in-place.
///
/// Modifies `current` directly by merging the patch.
/// Fields present in the patch replace the corresponding fields.
/// `null` in the patch removes the field (if the target type supports it, e.g. `Option<T>`).
/// Absent fields remain unchanged.
///
/// The patch can be any type that implements `AsRef<[u8]>` (`&str`, `String`, `Vec<u8>`, `&[u8]`, etc.).
///
/// # Errors
///
/// Returns an error if serialization, deserialization, or patch parsing fails.
#[allow(dead_code)]
pub fn apply_merge_patch_mut<T, P>(current: &mut T, patch: P) -> Result<(), serde_json::Error>
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

/// Applies a JSON Merge Patch (RFC 7396) in-place.
///
/// Modifies the current value directly.
///
/// # Example
///
/// ```
/// use serde_patch::apply_mut;
///
/// #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
/// struct User { id: u32, name: String }
///
/// let mut user = User { id: 1, name: "old".to_string() };
/// let patch = r#"{ "name": "new" }"#;
///
/// apply_mut!(&mut user, patch).unwrap();
/// assert_eq!(user.name, "new");
/// assert_eq!(user.id, 1);
/// ```
#[macro_export]
macro_rules! apply_mut {
    ($current:expr, $patch:expr) => {{ $crate::apply_patch_mut::apply_merge_patch_mut($current, $patch) }};
}
