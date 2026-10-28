#![cfg(feature = "rev-map")]

use mime_infer::{get_extensions, get_mime_extensions, get_mime_extensions_str};

#[test]
fn reverse_lookup_preserves_case_spaces_and_ignored_parameters() {
    let expected = get_extensions("text", "plain").unwrap();
    for input in [
        "text/plain",
        "TEXT/PLAIN",
        " text/plain ; charset=utf-8 ",
        "text/plain;not-a-valid-parameter",
    ] {
        assert_eq!(get_mime_extensions_str(input), Some(expected), "{input:?}");
    }
    assert_eq!(get_extensions("TEXT", "PLAIN"), Some(expected));
}

#[test]
fn reverse_lookup_rejects_invalid_essences() {
    for input in [
        "",
        "text",
        "/plain",
        "text/",
        "text/plain/extra",
        "*/plain",
        "*/bogus/",
        "text/pl*ain",
        "te*xt/plain",
        "**/*",
        "*/*extra",
        "text/pläin",
        "application/jaVAſcript",
        "text/plain\u{2003}",
        "\u{2003}text/plain",
        "text/pl ain",
        "text /plain",
        "text/ plain",
        "text/(plain)",
        "text/plain?",
    ] {
        assert_eq!(get_mime_extensions_str(input), None, "{input:?}");
    }
    for (top, sub) in [
        ("", "plain"),
        ("text", ""),
        ("*", "plain"),
        ("*", "bogus/"),
        ("text", "plain/extra"),
        ("te*xt", "plain"),
        ("text", "pl*ain"),
        ("application", "jaVAſcript"),
        (" text", "plain"),
        ("text", "plain "),
    ] {
        assert_eq!(get_extensions(top, sub), None, "{top:?}/{sub:?}");
    }
}

#[test]
fn reverse_lookup_rejects_every_ascii_control_character() {
    for byte in (0..=31).chain(std::iter::once(127)) {
        let control = char::from(byte);
        for input in [
            format!("{control}text/plain"),
            format!("text/pl{control}ain"),
            format!("*/{control}"),
            format!("text/plain;ignored={control}"),
        ] {
            assert_eq!(get_mime_extensions_str(&input), None, "{input:?}");
        }
        assert_eq!(get_extensions("*", &control.to_string()), None);
        assert_eq!(get_extensions(&format!("text{control}"), "plain"), None);
        assert_eq!(get_extensions("text", &format!("plain{control}")), None);
    }
}

#[test]
fn reverse_lookup_supports_only_complete_wildcards() {
    let all = get_extensions("*", "*").unwrap();
    let text = get_extensions("text", "*").unwrap();
    assert!(!all.is_empty());
    assert!(!text.is_empty());
    assert!(text.iter().all(|extension| all.contains(extension)));
    assert_eq!(get_mime_extensions_str("*/*"), Some(all));
    assert_eq!(get_mime_extensions_str("TEXT/*;ignored"), Some(text));
    assert_eq!(get_mime_extensions(&"*/*".parse().unwrap()), Some(all));
    assert_eq!(get_mime_extensions(&"text/*".parse().unwrap()), Some(text));
    assert_eq!(get_mime_extensions(&"*/plain".parse().unwrap()), None);
}
