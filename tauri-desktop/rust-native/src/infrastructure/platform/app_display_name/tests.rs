use super::*;

#[test]
fn unresolved_bundle_ids_are_none() {
    let resolver = UnsupportedAppDisplayNameResolver;
    for bundle_id in ["", "com.unknown.app", "com.apple.Terminal"] {
        assert_eq!(resolver.resolve(bundle_id), None, "bundle_id={bundle_id:?}");
    }
}
