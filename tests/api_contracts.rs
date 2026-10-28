use mime_infer::{Mime, MimeInfer, from_ext, from_path};

#[path = "../src/mime_types.rs"]
mod data;

#[test]
fn every_extension_matches_the_table_in_both_cases() {
    for &(ext, types) in data::MIME_TYPES {
        assert!(!ext.is_empty());
        assert!(ext.is_ascii());
        assert_eq!(ext, ext.to_ascii_lowercase());
        for input in [ext.to_owned(), ext.to_ascii_uppercase()] {
            let inferred = from_ext(&input);
            assert_eq!(inferred.iter_raw().collect::<Vec<_>>(), types, "{input}");
            assert_eq!(inferred.len(), types.len());
            assert_eq!(inferred.count(), types.len());
            assert!(!inferred.is_empty());
        }
    }
}

#[cfg(feature = "rev-map")]
#[test]
fn every_string_reverse_mapping_contains_its_extension() {
    for &(ext, types) in data::MIME_TYPES {
        for media_type in types {
            let exts = mime_infer::get_mime_extensions_str(media_type).unwrap();
            assert!(exts.contains(&ext), "{ext}: {media_type}");
        }
    }
}

#[test]
fn iterators_keep_their_length_when_consumed_from_both_ends() {
    for &(ext, types) in data::MIME_TYPES {
        let inferred = from_ext(ext);
        let mut raw = inferred.iter_raw();
        let mut parsed = inferred.iter();
        let mut remaining: std::collections::VecDeque<_> = types.iter().copied().collect();
        assert_eq!(raw.size_hint(), (types.len(), Some(types.len())));
        assert_eq!(parsed.size_hint(), raw.size_hint());
        for index in 0..types.len() {
            if index % 2 == 0 {
                let expected = remaining.pop_front().unwrap();
                assert_eq!(raw.next(), Some(expected));
                assert_eq!(parsed.next(), Some(expected.parse::<Mime>().unwrap()));
            } else {
                let back = remaining.pop_back().unwrap();
                assert_eq!(raw.next_back(), Some(back));
                assert_eq!(parsed.next_back(), Some(back.parse::<Mime>().unwrap()));
            }
            assert_eq!(raw.len(), types.len() - index - 1);
            assert_eq!(parsed.len(), raw.len());
            assert_eq!(raw.size_hint(), (raw.len(), Some(raw.len())));
        }
        assert_eq!(raw.next(), None);
        assert_eq!(raw.next_back(), None);
        assert_eq!(parsed.next(), None);
        assert_eq!(parsed.next_back(), None);
        let expected: Vec<Mime> = types.iter().map(|s| s.parse().unwrap()).collect();
        assert_eq!(inferred.into_iter().collect::<Vec<_>>(), expected);
        assert_eq!((&inferred).into_iter().collect::<Vec<_>>(), expected);
    }
}

#[test]
fn default_and_fallbacks_preserve_the_empty_contract() {
    for inferred in [
        MimeInfer::default(),
        from_ext(""),
        from_ext("unknown-extension"),
    ] {
        assert!(inferred.is_empty());
        assert_eq!(inferred.first(), None);
        assert_eq!(inferred.first_raw(), None);
        assert_eq!(inferred.iter().len(), 0);
        assert_eq!(
            inferred.first_or_octet_stream(),
            mime_infer::mime::APPLICATION_OCTET_STREAM
        );
        assert_eq!(inferred.first_or_text_plain(), mime_infer::mime::TEXT_PLAIN);
        assert_eq!(
            inferred.first_or_else(|| mime_infer::mime::IMAGE_GIF),
            mime_infer::mime::IMAGE_GIF
        );
    }
    assert_eq!(
        from_ext("txt").first_or_else(|| panic!("fallback must be lazy")),
        mime_infer::mime::TEXT_PLAIN
    );
}

#[test]
fn paths_follow_native_extension_rules() {
    for input in ["file", ".gitignore", "file.", "", "folder.txt/file"] {
        assert!(from_path(input).is_empty(), "{input}");
    }
    for input in ["file.TXT", ".config.txt", "archive.tar.txt"] {
        assert_eq!(from_path(input).first_raw(), Some("text/plain"), "{input}");
    }
    let nested = std::path::Path::new("folder.json").join("file.txt");
    assert_eq!(from_path(nested).first_raw(), Some("text/plain"));
    #[cfg(windows)]
    assert_eq!(
        from_path(r"C:\folder.json\file.TXT").first_raw(),
        Some("text/plain")
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_unix_extensions_are_unknown() {
    use std::os::unix::ffi::OsStringExt;
    let path = std::ffi::OsString::from_vec(b"file.\xff".to_vec());
    assert!(from_path(path).is_empty());
    let path = std::ffi::OsString::from_vec(b"\xff.txt".to_vec());
    assert_eq!(from_path(path).first_raw(), Some("text/plain"));
}

#[cfg(windows)]
#[test]
fn non_utf8_windows_extensions_are_unknown() {
    use std::os::windows::ffi::OsStringExt;
    let path = std::ffi::OsString::from_wide(&[b'f' as u16, b'.' as u16, 0xd800]);
    assert!(from_path(path).is_empty());
    let path = std::ffi::OsString::from_wide(&[
        0xd800,
        b'.' as u16,
        b't' as u16,
        b'x' as u16,
        b't' as u16,
    ]);
    assert_eq!(from_path(path).first_raw(), Some("text/plain"));
}

#[test]
fn varied_utf8_inputs_do_not_panic() {
    let alphabet = [
        'a', 'Z', '0', '.', '/', '\\', '\0', '\r', '\n', '*', ';', 'ſ', 'K', '中', '💾',
    ];
    let mut state = 0x4d49_4d45_u64;
    for length in 0..128 {
        let mut input = String::new();
        for _ in 0..length {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            input.push(alphabet[(state >> 32) as usize % alphabet.len()]);
        }
        let _ = from_ext(&input).iter().collect::<Vec<_>>();
        let _ = from_path(&input).iter().collect::<Vec<_>>();
        #[cfg(feature = "rev-map")]
        {
            let _ = mime_infer::get_mime_extensions_str(&input);
            let _ = mime_infer::get_extensions(&input, &input);
        }
    }
}
