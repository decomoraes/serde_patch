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
    println!("Basic patch (no forced fields):\n{}", basic_patch);

    // Diff with forced field – includes "id" even though unchanged
    let forced_patch = serde_json::to_string(&serde_patch::diff_including(&old, &new, &["id"])?)?;
    println!("\nPatch with forced \"id\":\n{}", forced_patch);

    // Apply immutably
    let updated = serde_patch::apply(old.clone(), &basic_patch)?;
    println!("\nImmutable apply result:\n{:#?}", updated);

    // Apply mutably
    let mut current = old;
    serde_patch::apply_mut(&mut current, &forced_patch)?;
    println!("\nMutable apply result:\n{:#?}", current);

    assert_eq!(updated, new);
    assert_eq!(current, new);

    Ok(())
}
