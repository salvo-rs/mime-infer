use mime_infer::from_ext;

const UPDATED_MAPPINGS: &[(&str, &[&str])] = &[
    ("js", &["text/javascript", "application/javascript"]),
    ("jsm", &["text/javascript", "application/javascript"]),
    ("mjs", &["text/javascript", "application/javascript"]),
    ("yaml", &["application/yaml", "text/x-yaml"]),
    ("yml", &["application/yaml", "text/x-yaml"]),
    ("woff", &["font/woff", "application/font-woff"]),
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
            assert!(
                mime_infer::get_mime_extensions_str(media_type)
                    .unwrap()
                    .contains(&ext),
                "{media_type} must include {ext}"
            );
        }
    }
}
