mod apply_patch;
mod apply_patch_mut;
mod diff_patch;

pub use apply_patch::apply;
pub use apply_patch_mut::apply_mut;
pub use diff_patch::diff;
pub use diff_patch::diff_including;

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Profile {
        bio: String,
        avatar_url: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct User {
        id: u32,
        username: String,
        age: u8,
        active: bool,
        profile: Option<Profile>,
    }

    #[test]
    fn test_diff() {
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

        let patch = serde_json::to_string(&crate::diff(&old, &new).unwrap()).unwrap();

        assert_eq!(
            patch,
            r#"{"active":false,"age":31,"profile":{"avatar_url":null,"bio":"Senior software engineer"}}"#
        );
    }

    #[test]
    fn test_diff_forced_nested() {
        let old = User {
            id: 1001,
            username: "alice".to_string(),
            age: 30,
            active: true,
            profile: Some(Profile {
                bio: "Software engineer".to_string(), // igual em new
                avatar_url: Some("https://example.com/alice-old.jpg".to_string()),
            }),
        };
        let new = User {
            id: 1001,
            username: "alice".to_string(),
            age: 31,
            active: false,
            profile: Some(Profile {
                bio: "Software engineer".to_string(),
                avatar_url: None,
            }),
        };

        let patch_value = crate::diff_including(&old, &new, &["profile.bio"]).unwrap();

        let expected = json!({
            "age": 31,
            "active": false,
            "profile": {
                "bio": "Software engineer",
                "avatar_url": null
            }
        });

        assert_eq!(patch_value, expected);
    }

    #[test]
    fn test_apply_patch_immutable() {
        let current = User {
            id: 1001,
            username: "alice".to_string(),
            age: 30,
            active: true,
            profile: Some(Profile {
                bio: "Software engineer".to_string(),
                avatar_url: Some("https://example.com/alice-old.jpg".to_string()),
            }),
        };

        let patch = r#"
            {
                "age": 31,
                "active": false,
                "profile": {
                    "bio": "Senior software engineer",
                    "avatar_url": null
                }
            }
        "#;

        let updated: User = crate::apply(current, patch).unwrap();

        assert_eq!(
            updated,
            User {
                id: 1001,
                username: "alice".to_string(),
                age: 31,
                active: false,
                profile: Some(Profile {
                    bio: "Senior software engineer".to_string(),
                    avatar_url: None,
                }),
            }
        );
    }

    #[test]
    fn test_apply_patch_mutable() {
        let mut current = User {
            id: 1001,
            username: "alice".to_string(),
            age: 30,
            active: true,
            profile: Some(Profile {
                bio: "Software engineer".to_string(),
                avatar_url: Some("https://example.com/alice-old.jpg".to_string()),
            }),
        };

        let patch = r#"
            {
                "age": 31,
                "active": false,
                "profile": {
                    "bio": "Senior software engineer",
                    "avatar_url": null
                }
            }
        "#;

        crate::apply_mut(&mut current, patch).unwrap();

        assert_eq!(
            current,
            User {
                id: 1001,
                username: "alice".to_string(),
                age: 31,
                active: false,
                profile: Some(Profile {
                    bio: "Senior software engineer".to_string(),
                    avatar_url: None,
                }),
            }
        );
    }
}
