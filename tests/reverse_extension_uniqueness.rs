#![cfg(feature = "rev-map")]

#[path = "../src/mime_types.rs"]
mod mime_types;

use mime_infer::{get_extensions, get_mime_extensions_str};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn global_wildcard_contains_every_extension_once() {
    let expected: BTreeSet<_> = mime_types::MIME_TYPES.iter().map(|&(ext, _)| ext).collect();
    let actual = get_extensions("*", "*").unwrap();
    assert_eq!(actual.len(), expected.len());
    assert!(actual.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(actual.iter().copied().collect::<BTreeSet<_>>(), expected);
    assert_eq!(get_mime_extensions_str("*/*"), Some(actual));
}

#[test]
fn every_top_level_wildcard_contains_its_extensions_once() {
    let mut expected: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for &(ext, types) in mime_types::MIME_TYPES {
        for media_type in types {
            let (top, _) = media_type.split_once('/').unwrap();
            expected.entry(top).or_default().insert(ext);
        }
    }
    for (top, extensions) in expected {
        let actual = get_extensions(top, "*").unwrap();
        assert_eq!(actual.len(), extensions.len(), "{top}/* has duplicates");
        assert!(
            actual.windows(2).all(|pair| pair[0] < pair[1]),
            "{top}/* is not in lexicographic order"
        );
        assert_eq!(
            actual.iter().copied().collect::<BTreeSet<_>>(),
            extensions,
            "{top}/*"
        );
    }
}

#[test]
fn exact_reverse_lookups_keep_every_alias_and_their_order() {
    let mut expected: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for &(ext, types) in mime_types::MIME_TYPES {
        for media_type in types {
            expected.entry(media_type).or_default().push(ext);
        }
    }
    for (media_type, extensions) in expected {
        assert_eq!(
            get_mime_extensions_str(media_type).unwrap(),
            extensions,
            "{media_type}"
        );
    }
}
