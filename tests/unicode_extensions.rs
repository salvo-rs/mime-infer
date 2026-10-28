use mime_infer::{MimeInfer, from_ext, from_path};

#[test]
fn unicode_case_folding_matches_ascii_extensions() {
    for (unicode, ascii) in [
        ("jſ", "js"),
        ("ſvg", "svg"),
        ("ſhtml", "shtml"),
        ("Kml", "kml"),
    ] {
        let expected = from_ext(ascii);
        assert!(
            !expected.is_empty(),
            "expected a known mapping for {ascii:?}"
        );
        assert_eq!(from_ext(unicode), expected, "{unicode:?}");
        assert_eq!(MimeInfer::from_ext(unicode), expected, "{unicode:?}");
        assert_eq!(
            from_path(format!("file.{unicode}")),
            expected,
            "{unicode:?}"
        );
        assert_eq!(
            MimeInfer::from_path(format!("file.{unicode}")),
            expected,
            "{unicode:?}"
        );
    }
}

#[test]
fn unicode_case_folding_does_not_match_arbitrary_lookalikes() {
    assert!(from_ext("ｓｖｇ").is_empty());
    assert!(from_path("file.ｓｖｇ").is_empty());
}
