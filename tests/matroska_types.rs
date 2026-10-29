use mime_infer::{Mime, from_ext};

// Fixed expectations derived from the official registrations below.
const CASES: &[(&str, &[&str])] = &[
    // https://www.iana.org/assignments/media-types/video/matroska-3d
    // Keep the registered historical 3D alias and the existing generic alias.
    (
        "mk3d",
        &[
            "video/matroska-3d",
            "video/x-matroska-3d",
            "video/x-matroska",
        ],
    ),
    // https://www.iana.org/assignments/media-types/audio/matroska
    ("mka", &["audio/matroska", "audio/x-matroska"]),
    // https://www.iana.org/assignments/media-types/video/matroska
    ("mkv", &["video/matroska", "video/x-matroska"]),
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
