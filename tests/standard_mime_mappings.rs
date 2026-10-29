use mime_infer::from_ext;

const UPDATED_MAPPINGS: &[(&str, &[&str])] = &[
    // RFC 9239: https://www.rfc-editor.org/rfc/rfc9239.html#section-6.1.1
    ("js", &["text/javascript", "application/javascript"]),
    ("jsm", &["text/javascript", "application/javascript"]),
    ("mjs", &["text/javascript", "application/javascript"]),
    // RFC 9512: https://www.rfc-editor.org/rfc/rfc9512.html#section-2.1
    ("yaml", &["application/yaml", "text/x-yaml"]),
    ("yml", &["application/yaml", "text/x-yaml"]),
    // RFC 8081: https://www.rfc-editor.org/rfc/rfc8081.html#section-4.4.5
    ("woff", &["font/woff", "application/font-woff"]),
    // RFC 5545: https://www.rfc-editor.org/rfc/rfc5545.html#section-8.1
    // Calendar files have a specific registered type rather than a binary fallback.
    ("ics", &["text/calendar"]),
    // RFC 6350: https://www.iana.org/assignments/media-types/text/vcard
    ("vcf", &["text/vcard", "text/x-vcard"]),
    // RFC 8081: https://www.rfc-editor.org/rfc/rfc8081.html#section-4.4.3
    ("otf", &["font/otf", "application/font-sfnt"]),
    // Registration: https://www.iana.org/assignments/media-types/application/vnd.rar
    (
        "rar",
        &["application/vnd.rar", "application/x-rar-compressed"],
    ),
    // RFC 6922: https://www.rfc-editor.org/rfc/rfc6922.html#section-3
    ("sql", &["application/sql", "application/x-sql"]),
    // Registration: https://www.iana.org/assignments/media-types/application/toml
    ("toml", &["application/toml", "text/x-toml"]),
];

#[test]
fn current_standard_types_precede_preserved_aliases() {
    for &(ext, expected) in UPDATED_MAPPINGS {
        let inferred = from_ext(ext);
        assert_eq!(inferred.first_raw(), expected.first().copied(), "{ext}");
        assert_eq!(inferred.iter_raw().collect::<Vec<_>>(), expected, "{ext}");
        assert_eq!(inferred.first().unwrap().as_ref(), expected[0], "{ext}");
    }
}

#[test]
#[cfg(feature = "rev-map")]
fn standard_types_and_historical_aliases_both_reverse_lookup() {
    for &(ext, types) in UPDATED_MAPPINGS {
        for &media_type in types {
            let extensions = mime_infer::get_mime_extensions_str(media_type).unwrap();
            assert!(extensions.contains(&ext), "{media_type} must include {ext}");
            let parsed = media_type.parse().unwrap();
            assert_eq!(
                mime_infer::get_mime_extensions(&parsed),
                Some(extensions),
                "typed and string reverse lookups must agree for {media_type}"
            );
        }
    }
}
