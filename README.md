# serde-patch

JSON Merge Patch (RFC 7396) for Serde-derived types.

Provides macros to generate partial patches (diff) and apply them immutably or in-place.

```toml
[dependencies]
serde-patch = "0.2"
```

## Features

- `diff` – generates a minimal patch containing only changed fields.
- `diff_including` – generates a patch including specific fields even if unchanged.
- `apply` – consumes the current value and returns an updated one.
- `apply_mut` – modifies the current value in-place.

The patch can be any type that implements `AsRef<[u8]>` (`&str`, `String`, `Vec<u8>`, etc.).

## Example

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Profile {
    bio: String,
    avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: u32,
    username: String,
    age: u8,
    active: bool,
    profile: Option<Profile>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let old = User {
        id: 1001,
        username: "alice".to_string(),
        age: 30,
        active: true,
        profile: Some(Profile {
            bio: "Software engineer".to_string(),
            avatar_url: Some("https://example.com/alice-old.jpg".to_string()),
        }),
    };

    let new = User {
        id: 1001,
        username: "alice".to_string(),
        age: 31,
        active: false,
        profile: Some(Profile {
            bio: "Senior software engineer".to_string(),
            avatar_url: None,
        }),
    };

    // Basic diff – only changed fields
    let basic_patch = serde_json::to_string(&serde_patch::diff(&old, &new)?)?;
    // → {"active":false,"age":31,"profile":{"avatar_url":null,"bio":"Senior software engineer"}}

    // Diff with forced field – includes "id" and "profile.bio" even though unchanged
    let forced_patch = serde_json::to_string(&serde_patch::diff_including(&old, &new, &["id", "profile.bio"])?)?;
    // → {"active":false,"age":31,"id":1001,"profile":{"avatar_url":null,"bio":"Senior software engineer"}}

    // Apply immutably
    let updated = serde_patch::apply(old.clone(), &basic_patch)?;
    assert_eq!(updated, new);

    // Apply mutably
    let mut current = old;
    serde_patch::apply_mut(&mut current, &forced_patch)?;
    assert_eq!(current, new);

    Ok(())
}
```

## Functions

- `diff(&old, &new)` – basic diff (only changed fields).
- `diff_including(&old, &new, &["path.to.field", ...])` – include forced fields even if unchanged.
- `apply(current, &patch)` – immutable.
- `apply_mut(&mut current, &patch)` – mutable.
