use mime_infer::{Mime, from_ext};

// Fixed expectations derived from the official registrations below.
const CASES: &[(&str, &[&str])] = &[
    // https://www.iana.org/assignments/media-types/application/wsdl+xml
    ("wsdl", &["application/wsdl+xml", "text/xml"]),
    // https://www.iana.org/assignments/media-types/application/xliff+xml
    ("xlf", &["application/xliff+xml", "application/x-xliff+xml"]),
    // https://www.iana.org/assignments/media-types/application/xslt+xml
    ("xsl", &["application/xslt+xml", "text/xml"]),
    // https://www.iana.org/assignments/media-types/application/xslt+xml
    ("xslt", &["application/xslt+xml", "text/xml"]),
    // https://www.iana.org/assignments/media-types/image/vnd.microsoft.icon
    ("ico", &["image/vnd.microsoft.icon", "image/x-icon"]),
    // https://www.iana.org/assignments/media-types/image/emf
    ("emf", &["image/emf", "application/x-msmetafile"]),
    // https://www.iana.org/assignments/media-types/image/wmf
    ("wmf", &["image/wmf", "application/x-msmetafile"]),
    // https://www.iana.org/assignments/media-types/image/vnd.dwg
    ("dwg", &["image/vnd.dwg", "application/acad"]),
    // https://www.iana.org/assignments/media-types/image/vnd.dxf
    ("dxf", &["image/vnd.dxf", "application/x-dxf"]),
    // https://www.iana.org/assignments/media-types/model/step
    ("step", &["model/step", "application/step"]),
    // https://www.iana.org/assignments/media-types/model/step
    ("stp", &["model/step", "application/step"]),
    // https://www.iana.org/assignments/media-types/model/vnd.dwf
    ("dwf", &["model/vnd.dwf", "drawing/x-dwf"]),
    // https://www.iana.org/assignments/media-types/model/x3d+fastinfoset
    ("x3db", &["model/x3d+fastinfoset", "model/x3d+binary"]),
    // https://www.iana.org/assignments/media-types/model/x3d-vrml
    ("x3dv", &["model/x3d-vrml", "model/x3d+vrml"]),
    // https://www.iana.org/assignments/media-types/application/vnd.smaf
    ("mmf", &["application/vnd.smaf", "application/x-smaf"]),
    // https://www.iana.org/assignments/media-types/application/vnd.apple.mpegurl
    // HLS is a specialized use of M3U8; retain the general playlist default.
    (
        "m3u8",
        &["audio/x-mpegurl", "application/vnd.apple.mpegurl"],
    ),
    // https://www.iana.org/assignments/media-types/application/pkix-cert
    (
        "cer",
        &["application/pkix-cert", "application/x-x509-ca-cert"],
    ),
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
