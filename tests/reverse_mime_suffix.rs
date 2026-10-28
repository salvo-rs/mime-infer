#![cfg(feature = "rev-map")]

#[path = "../src/mime_types.rs"]
mod mime_types;

use mime_infer::{Mime, get_mime_extensions, get_mime_extensions_str};

#[test]
fn typed_reverse_lookup_matches_every_registered_media_type() {
    for &(ext, types) in mime_types::MIME_TYPES {
        for &media_type in types {
            let mime: Mime = media_type.parse().unwrap();
            let expected = get_mime_extensions_str(media_type).unwrap();
            assert!(expected.contains(&ext), "{media_type} must include {ext}");
            assert_eq!(get_mime_extensions(&mime), Some(expected), "{media_type}");
        }
    }
}

#[test]
fn structured_suffixes_preserve_case_and_parameter_behavior() {
    for (media_type, expected_ext) in [
        ("IMAGE/SVG+XML; charset=utf-8", "svg"),
        ("APPLICATION/LD+JSON; profile=example", "jsonld"),
    ] {
        let mime: Mime = media_type.parse().unwrap();
        let extensions = get_mime_extensions(&mime).unwrap();
        assert!(extensions.contains(&expected_ext));
        assert_eq!(get_mime_extensions_str(media_type), Some(extensions));
    }
}
