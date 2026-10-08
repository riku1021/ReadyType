use super::*;

struct StubIcons;
struct StubNames;

impl AppIconResolver for StubIcons {
    fn resolve(&self, bundle_id: &str) -> Option<String> {
        (bundle_id == "com.known.app").then(|| "data:image/png;base64,AAAA".to_owned())
    }
}

impl AppDisplayNameResolver for StubNames {
    fn resolve(&self, bundle_id: &str) -> Option<String> {
        (bundle_id == "com.known.app").then(|| "Known".to_owned())
    }
}

#[test]
fn maps_icon_and_display_name_together() {
    let ids = vec!["com.known.app".to_owned(), "com.unknown.app".to_owned()];
    let result = execute(&StubIcons, &StubNames, &ids);

    assert_eq!(
        result.get("com.known.app"),
        Some(&AppPresentation {
            icon: Some("data:image/png;base64,AAAA".to_owned()),
            display_name: Some("Known".to_owned()),
        })
    );
    assert_eq!(
        result.get("com.unknown.app"),
        Some(&AppPresentation {
            icon: None,
            display_name: None,
        })
    );
}
