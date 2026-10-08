use super::*;

#[test]
fn unresolved_bundle_ids_are_none() {
    let resolver = UnsupportedAppIconResolver;
    let source = UnsupportedAppIconSource;
    for bundle_id in ["", "com.unknown.app", "com.apple.Terminal"] {
        assert_eq!(resolver.resolve(bundle_id), None, "bundle_id={bundle_id:?}");
        assert_eq!(source.probe_mtime(bundle_id), None);
        assert_eq!(source.fetch(bundle_id), None);
    }
}
