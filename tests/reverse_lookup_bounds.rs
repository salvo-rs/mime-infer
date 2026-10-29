#![cfg(feature = "rev-map")]

#[path = "../src/mime_types.rs"]
mod mime_types;

use mime_infer::{get_extensions, get_mime_extensions_str};

#[test]
fn component_bounds_preserve_every_registered_reverse_mapping() {
    for &(_, media_types) in mime_types::MIME_TYPES {
        for &media_type in media_types {
            let (top, sub) = media_type.split_once('/').unwrap();
            let expected: Vec<_> = mime_types::MIME_TYPES
                .iter()
                .filter_map(|&(extension, types)| types.contains(&media_type).then_some(extension))
                .collect();
            assert_eq!(get_extensions(top, sub), Some(expected.as_slice()));
            assert_eq!(
                get_mime_extensions_str(media_type),
                Some(expected.as_slice())
            );
            assert_eq!(
                get_extensions(&top.to_ascii_uppercase(), &sub.to_ascii_uppercase()),
                Some(expected.as_slice())
            );
        }
    }
}

#[test]
fn overlong_components_and_unknown_essences_remain_unknown() {
    let long = "x".repeat(1_000_000);
    for (top, sub) in [
        (&*long, "plain"),
        ("text", &*long),
        ("*", &*long),
        (&*long, "*"),
    ] {
        assert_eq!(get_extensions(top, sub), None);
        assert_eq!(get_mime_extensions_str(&format!("{top}/{sub}")), None);
    }
    for input in [
        format!("unknown/plain;ignored={long}"),
        format!("text/unknown;ignored={long}"),
        format!("text/unknown {long}"),
        format!("text/{long};ignored=value"),
    ] {
        assert_eq!(get_mime_extensions_str(&input), None);
    }
}

#[test]
fn matching_essences_still_check_late_controls_and_ignore_parameter_syntax() {
    let parameter = "x".repeat(1_000_000);
    for essence in ["text/plain", "text/*", "*/*"] {
        let expected = get_mime_extensions_str(essence).unwrap();
        let input = format!("  {essence}  ;invalid-parameter={parameter}");
        assert_eq!(get_mime_extensions_str(&input), Some(expected));
        assert_eq!(get_mime_extensions_str(&format!("{input}\0")), None);
    }
}

#[test]
fn bounded_prefixes_preserve_spacing_and_utf8_boundaries() {
    for input in [
        "text/plain suffix;ignored",
        "text/plain /suffix",
        "text/ plain",
        "text /plain",
        "application/jaVAſcript",
        "text/plain\u{2003}",
    ] {
        assert_eq!(get_mime_extensions_str(input), None, "{input:?}");
    }
    let expected = get_mime_extensions_str("text/plain");
    assert_eq!(get_mime_extensions_str(" text/plain   "), expected);
    assert_eq!(get_mime_extensions_str("text/plain ;\u{2003}"), expected);

    // Cutoffs can land inside a multibyte scalar; bounded byte searches must
    // never turn those positions into invalid UTF-8 string slices.
    for padding in 0..128 {
        for input in [
            format!("{}😀/plain", "a".repeat(padding)),
            format!("text/{}😀;ignored", "a".repeat(padding)),
        ] {
            assert_eq!(get_mime_extensions_str(&input), None);
        }
    }
}
