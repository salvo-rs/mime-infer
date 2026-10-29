use mime_infer::{Mime, from_ext};

// Fixed expectations derived from the official registrations below.
const CASES: &[(&str, &[&str])] = &[
    // https://www.iana.org/assignments/media-types/image/jpm
    ("jpgm", &["image/jpm"]),
    // https://www.iana.org/assignments/media-types/image/vnd.dvb.subtitle
    ("sub", &["image/vnd.dvb.subtitle"]),
    // https://www.iana.org/assignments/media-types/application/vnd.ms-cab-compressed
    ("cab", &["application/vnd.ms-cab-compressed"]),
    // https://www.iana.org/assignments/media-types/image/vnd.zbrush.pcx
    ("pcx", &["image/vnd.zbrush.pcx"]),
    // https://www.iana.org/assignments/media-types/image/vnd.adobe.photoshop
    ("psd", &["image/vnd.adobe.photoshop"]),
    // https://www.iana.org/assignments/media-types/application/vnd.Quark.QuarkXPress
    ("qxd", &["application/vnd.quark.quarkxpress"]),
    // https://www.iana.org/assignments/media-types/application/xml-dtd
    ("dtd", &["application/xml-dtd"]),
];

#[test]
fn preferred_types_and_compatibility_aliases_match_the_registrations() {
    for &(ext, expected) in CASES {
        let inferred = from_ext(ext);
        assert_eq!(inferred.first_raw(), Some(expected[0]), "{ext}");
        assert_eq!(
            inferred.first(),
            Some(expected[0].parse::<Mime>().unwrap()),
            "{ext}"
        );
        assert_eq!(inferred.iter_raw().collect::<Vec<_>>(), expected, "{ext}");
    }
}

#[test]
#[cfg(feature = "rev-map")]
fn registered_types_and_retained_aliases_support_both_reverse_apis() {
    for &(ext, types) in CASES {
        for &media_type in types {
            let extensions = mime_infer::get_mime_extensions_str(media_type).unwrap();
            assert!(extensions.contains(&ext), "{media_type} must include {ext}");
            let parsed: Mime = media_type.parse().unwrap();
            assert_eq!(
                mime_infer::get_mime_extensions(&parsed),
                Some(extensions),
                "{media_type}"
            );
        }
    }
}

#[test]
#[cfg(feature = "rev-map")]
fn incorrect_types_and_binary_placeholders_no_longer_include_the_extensions() {
    for (ext, removed_type) in [
        ("jpgm", "video/jpm"),
        ("sub", "text/vnd.dvb.subtitle"),
        ("cab", "application/octet-stream"),
        ("pcx", "application/octet-stream"),
        ("psd", "application/octet-stream"),
        ("qxd", "application/octet-stream"),
        ("dtd", "text/xml"),
    ] {
        assert!(
            mime_infer::get_mime_extensions_str(removed_type)
                .is_none_or(|extensions| !extensions.contains(&ext)),
            "{removed_type} still includes {ext}"
        );
    }
}
