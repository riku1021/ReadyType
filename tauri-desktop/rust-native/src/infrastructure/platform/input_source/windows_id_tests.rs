use super::windows_id::{
    activate_profile_hresult_succeeded, format_windows_source_id, merge_windows_language_ids,
    parse_windows_source_id, WindowsSourceId,
};

#[test]
fn activate_profile_only_treats_s_ok_as_success() {
    assert!(activate_profile_hresult_succeeded(0));
    assert!(!activate_profile_hresult_succeeded(1));
    assert!(!activate_profile_hresult_succeeded(-2_147_467_259));
}

#[test]
fn windows_input_languages_include_tsf_languages_without_a_keyboard_layout() {
    assert_eq!(
        merge_windows_language_ids(&[0x0409], &[0x0411, 0x0409]),
        vec![0x0409, 0x0411]
    );
}

#[test]
fn windows_keyboard_layout_ids_round_trip_without_losing_the_hkl() {
    let source = WindowsSourceId::KeyboardLayout(0x0000_0000_0409_0409);

    let serialized = format_windows_source_id(&source);

    assert_eq!(serialized, "win:hkl:0000000004090409");
    assert_eq!(parse_windows_source_id(&serialized), Some(source));
}

#[test]
fn windows_text_service_ids_round_trip_profile_identity() {
    let source = WindowsSourceId::TextService {
        language_id: 0x0411,
        class_id: "A23A7C9E-94A4-4A2D-8B57-3D9E83D41F10".to_owned(),
        profile_id: "1F04A1F3-4C39-4F5A-A0B8-E9F304CE8D7A".to_owned(),
    };

    let serialized = format_windows_source_id(&source);

    assert_eq!(
        serialized,
        "win:tsf:0411:A23A7C9E-94A4-4A2D-8B57-3D9E83D41F10:1F04A1F3-4C39-4F5A-A0B8-E9F304CE8D7A"
    );
    assert_eq!(parse_windows_source_id(&serialized), Some(source));
}

#[test]
fn windows_source_id_parser_rejects_unknown_or_incomplete_values() {
    for invalid in [
        "",
        "win:hkl:not-hex",
        "win:tsf:0411:broken",
        "mac:com.apple.keylayout.ABC",
    ] {
        assert_eq!(parse_windows_source_id(invalid), None, "input={invalid:?}");
    }
}
