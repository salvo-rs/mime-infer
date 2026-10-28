//! Inferring MIME types by file extension.
//!
//! Uses a static list of file-extension : MIME type mappings.
//!
//! ```
//! // the file doesn't have to exist, it just looks at the path
//! let infer = mime_infer::from_path("some_file.gif");
//! assert_eq!(infer.first(), Some(mime::IMAGE_GIF));
//!
//! ```
//!
//! #### Note: MIME Types Returned Are Not Stable/Guaranteed
//! The media types returned for a given extension are not considered to be part of the crate's
//! stable API and are often updated in patch <br /> (`x.y.[z + 1]`) releases to be as correct as
//! possible.
//!
//! Additionally, only the extensions of paths/filenames are inspected in order to infer the MIME
//! type. The file that may or may not reside at that path may or may not be a valid file of the
//! returned MIME type.  Be wary of unsafe or un-validated assumptions about file structure or
//! length.
pub extern crate mime;

pub use mime::Mime;

use std::ffi::OsStr;
use std::iter::FusedIterator;
use std::path::Path;
use std::{iter, slice};

#[cfg(feature = "phf-map")]
#[path = "impl_phf.rs"]
mod impl_;

#[cfg(not(feature = "phf-map"))]
#[path = "impl_bin_search.rs"]
mod impl_;

/// An inference of the MIME/Media Type(s) of an extension or path as one or more
/// [`Mime`] instances.
///
/// ### Note: Ordering
/// A given file format may have one or more applicable Media Types; in this case
/// the first Media Type returned is whatever is declared in the latest IETF RFC for the
/// presumed file format or the one that explicitly supersedes all others.
/// Ordering of additional Media Types is arbitrary.
///
/// ### Note: Values Not Stable
/// The exact Media Types returned in any given infer are not considered to be stable and are often
/// updated in patch releases in order to reflect the most up-to-date information possible.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
// FIXME: change repr when `mime` gains macro/const fn constructor
pub struct MimeInfer(&'static [&'static str]);

impl MimeInfer {
    /// Infer the MIME type of a file (real or otherwise) with the given extension.
    ///
    /// The search uses Unicode case folding, so non-ASCII input can match an ASCII extension:
    /// for example, `jſ` matches `js` and `Kml` matches `kml`.
    /// Applications that require ASCII-only extensions must validate that requirement separately.
    ///
    /// If `ext` is empty or has no (currently) known MIME type mapping, then an empty infer is
    /// returned.
    pub fn from_ext(ext: &str) -> MimeInfer {
        if ext.is_empty() || ext.len() > impl_::MAX_EXT_INPUT_BYTES {
            return MimeInfer(&[]);
        }

        impl_::get_mime_types(ext).map_or(MimeInfer(&[]), MimeInfer)
    }

    /// Infer the MIME type of `path` by its extension (as defined by
    /// [`Path::extension()`]). **No disk access is performed.**
    ///
    /// If `path` has no extension, the extension cannot be converted to `str`, or has
    /// no known MIME type mapping, then an empty infer is returned.
    ///
    /// The extension is matched using the same Unicode case folding as [`Self::from_ext`].
    /// For example, `file.ſvg` matches the `svg` extension. Applications that require ASCII-only
    /// filenames or extensions must validate that requirement separately.
    ///
    /// ## Note
    /// **Infer** is the operative word here, as there are no guarantees that the contents of the
    /// file that `path` points to match the MIME type associated with the path's extension.
    ///
    /// Take care when processing files with assumptions based on the return value of this function.
    ///
    /// [`Path::extension()`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.extension
    pub fn from_path<P: AsRef<Path>>(path: P) -> MimeInfer {
        path.as_ref()
            .extension()
            .and_then(OsStr::to_str)
            .map_or(MimeInfer(&[]), Self::from_ext)
    }

    /// `true` if the infer did not return any known mappings for the given path or extension.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of MIME types in the current inference.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get the number of MIME types in the current inference.
    pub fn count(&self) -> usize {
        self.len()
    }

    /// Get the first inferred `Mime`, if applicable.
    ///
    /// Each call parses the stored media-type string. If only the string is
    /// needed, use [`Self::first_raw()`] to avoid parsing on a hot path.
    ///
    /// See [Note: Ordering](#note-ordering) above.
    pub fn first(&self) -> Option<Mime> {
        self.first_raw().map(expect_mime)
    }

    /// Get the first inferred Media Type as a string, if applicable.
    ///
    /// See [Note: Ordering](#note-ordering) above.
    pub fn first_raw(&self) -> Option<&'static str> {
        self.0.first().copied()
    }

    /// Get the first inferred `Mime`, or if the infer is empty, return
    /// [`application/octet-stream`] instead.
    ///
    /// See [Note: Ordering](#note-ordering) above.
    ///
    /// ### Note: HTTP Applications
    /// For HTTP request and response bodies if a value for the `Content-Type` header
    /// cannot be determined it might be preferable to not send one at all instead of defaulting to
    /// `application/octet-stream` as the recipient will expect to infer the format directly from
    /// the content instead. ([RFC 7231, Section 3.1.1.5][rfc7231])
    ///
    /// On the contrary, for `multipart/form-data` bodies, the `Content-Type` of a form-data part is
    /// assumed to be `text/plain` unless specified so a default of `application/octet-stream`
    /// for non-text parts is safer. ([RFC 7578, Section 4.4][rfc7578])
    ///
    /// [`application/octet-stream`]: https://docs.rs/mime/0.3/mime/constant.APPLICATION_OCTET_STREAM.html
    /// [rfc7231]: https://tools.ietf.org/html/rfc7231#section-3.1.1.5
    /// [rfc7578]: https://tools.ietf.org/html/rfc7578#section-4.4
    pub fn first_or_octet_stream(&self) -> Mime {
        self.first_or(mime::APPLICATION_OCTET_STREAM)
    }

    /// Get the first inferred `Mime`, or if the infer is empty, return
    /// [`text/plain`](::mime::TEXT_PLAIN) instead.
    ///
    /// See [Note: Ordering](#note-ordering) above.
    pub fn first_or_text_plain(&self) -> Mime {
        self.first_or(mime::TEXT_PLAIN)
    }

    /// Get the first inferred `Mime`, or if the infer is empty, return the given `Mime` instead.
    ///
    /// See [Note: Ordering](#note-ordering) above.
    pub fn first_or(&self, default: Mime) -> Mime {
        self.first().unwrap_or(default)
    }

    /// Get the first inferred `Mime`, or if the infer is empty, execute the closure and return its
    /// result.
    ///
    /// See [Note: Ordering](#note-ordering) above.
    pub fn first_or_else<F>(&self, default_fn: F) -> Mime
    where
        F: FnOnce() -> Mime,
    {
        self.first().unwrap_or_else(default_fn)
    }

    /// Get an iterator over the `Mime` values contained in this infer.
    ///
    /// Each yielded value is parsed from its stored media-type string. Use
    /// [`Self::iter_raw()`] when the strings themselves are sufficient.
    ///
    /// See [Note: Ordering](#note-ordering) above.
    pub fn iter(&self) -> Iter {
        Iter(self.iter_raw().map(expect_mime))
    }

    /// Get an iterator over the raw media-type strings in this infer.
    ///
    /// See [Note: Ordering](#note-ordering) above.
    pub fn iter_raw(&self) -> IterRaw {
        IterRaw(self.0.iter().cloned())
    }
}

impl IntoIterator for MimeInfer {
    type Item = Mime;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for &MimeInfer {
    type Item = Mime;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the `Mime` types of a `MimeInfer`.
///
/// See [Note: Ordering on `MimeInfer`](struct.MimeInfer.html#note-ordering).
#[derive(Clone, Debug)]
pub struct Iter(iter::Map<IterRaw, fn(&'static str) -> Mime>);

impl Iterator for Iter {
    type Item = Mime;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl FusedIterator for Iter {}

impl ExactSizeIterator for Iter {
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// An iterator over the raw media type strings of a `MimeInfer`.
///
/// See [Note: Ordering on `MimeInfer`](struct.MimeInfer.html#note-ordering).
#[derive(Clone, Debug)]
pub struct IterRaw(iter::Cloned<slice::Iter<'static, &'static str>>);

impl Iterator for IterRaw {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for IterRaw {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl FusedIterator for IterRaw {}

impl ExactSizeIterator for IterRaw {
    fn len(&self) -> usize {
        self.0.len()
    }
}

fn expect_mime(s: &str) -> Mime {
    // `.parse()` should be checked at compile time to never fail
    s.parse()
        .unwrap_or_else(|e| panic!("failed to parse media-type {:?}: {}", s, e))
}

/// Wrapper of [`MimeInfer::from_ext()`](struct.MimeInfer.html#method.from_ext).
pub fn from_ext(ext: &str) -> MimeInfer {
    MimeInfer::from_ext(ext)
}

/// Wrapper of [`MimeInfer::from_path()`](struct.MimeInfer.html#method.from_path).
pub fn from_path<P: AsRef<Path>>(path: P) -> MimeInfer {
    MimeInfer::from_path(path)
}

/// Infer the MIME type of `path` by its extension (as defined by `Path::extension()`).
///
/// If `path` has no extension, or its extension has no known MIME type mapping,
/// then the MIME type is assumed to be `application/octet-stream`.
///
/// ## Note
/// **Infer** is the operative word here, as there are no guarantees that the contents of the file
/// that `path` points to match the MIME type associated with the path's extension.
///
/// Take care when processing files with assumptions based on the return value of this function.
///
/// In HTTP applications, it might be [preferable][rfc7231] to not send a `Content-Type`
/// header at all instead of defaulting to `application/octet-stream`.
///
/// [rfc7231]: https://tools.ietf.org/html/rfc7231#section-3.1.1.5
#[deprecated(
    since = "2.0.0",
    note = "Use `from_path(path).first_or_octet_stream()` instead"
)]
pub fn infer_mime_type<P: AsRef<Path>>(path: P) -> Mime {
    from_path(path).first_or_octet_stream()
}

/// Infer the MIME type of `path` by its extension (as defined by `Path::extension()`).
///
/// If `path` has no extension, or its extension has no known MIME type mapping,
/// then `None` is returned.
///
#[deprecated(since = "2.0.0", note = "Use `from_path(path).first()` instead")]
pub fn infer_mime_type_opt<P: AsRef<Path>>(path: P) -> Option<Mime> {
    from_path(path).first()
}

/// Infer the MIME type string of `path` by its extension (as defined by `Path::extension()`).
///
/// If `path` has no extension, or its extension has no known MIME type mapping,
/// then `None` is returned.
///
/// ## Note
/// **Infer** is the operative word here, as there are no guarantees that the contents of the file
/// that `path` points to match the MIME type associated with the path's extension.
///
/// Take care when processing files with assumptions based on the return value of this function.
#[deprecated(since = "2.0.0", note = "Use `from_path(path).first_raw()` instead")]
pub fn mime_str_for_path_ext<P: AsRef<Path>>(path: P) -> Option<&'static str> {
    from_path(path).first_raw()
}

/// Get the MIME type associated with a file extension.
///
/// If there is no association for the extension, or `ext` is empty,
/// `application/octet-stream` is returned.
///
/// ## Note
/// In HTTP applications, it might be [preferable][rfc7231] to not send a `Content-Type`
/// header at all instead of defaulting to `application/octet-stream`.
///
/// [rfc7231]: https://tools.ietf.org/html/rfc7231#section-3.1.1.5
#[deprecated(
    since = "2.0.0",
    note = "use `from_ext(search_ext).first_or_octet_stream()` instead"
)]
pub fn get_mime_type(search_ext: &str) -> Mime {
    from_ext(search_ext).first_or_octet_stream()
}

/// Get the MIME type associated with a file extension.
///
/// If there is no association for the extension, or `ext` is empty,
/// `None` is returned.
#[deprecated(since = "2.0.0", note = "use `from_ext(search_ext).first()` instead")]
pub fn get_mime_type_opt(search_ext: &str) -> Option<Mime> {
    from_ext(search_ext).first()
}

/// Get the MIME type string associated with a file extension. Case-insensitive.
///
/// If `search_ext` is not already lowercase,
/// it will be converted to lowercase to facilitate the search.
///
/// Returns `None` if `search_ext` is empty or an associated extension was not found.
#[deprecated(
    since = "2.0.0",
    note = "use `from_ext(search_ext).first_raw()` instead"
)]
pub fn get_mime_type_str(search_ext: &str) -> Option<&'static str> {
    from_ext(search_ext).first_raw()
}

/// Get a list of known extensions for a given `Mime`.
///
/// Ignores parameters (only searches with `<main type>/<subtype>`). Case-insensitive (for extension types).
///
/// Returns `None` if the MIME type is unknown.
///
/// ### Wildcards
/// `*/*` returns all extensions. A wildcard top-level with a concrete subtype is invalid.
///
/// If the sub-level of the MIME type is a wildcard, returns all extensions for the top-level.
///
/// Wildcard results contain each extension once, in lexicographic order.
#[cfg(feature = "rev-map")]
pub fn get_mime_extensions(mime: &Mime) -> Option<&'static [&'static str]> {
    get_mime_extensions_str(mime.essence_str())
}

/// Get a list of known extensions for a MIME type string.
///
/// Ignores parameters (only searches `<main type>/<subtype>`). Case-insensitive.
/// ASCII spaces surrounding the essence are ignored; the type and subtype must be nonempty ASCII
/// tokens with no surrounding spaces of their own. ASCII control characters are rejected anywhere
/// in the input. Parameters are otherwise not validated.
///
/// Returns `None` if the MIME type is invalid or unknown.
///
/// ### Wildcards
/// `*/*` returns all extensions. A wildcard top-level with a concrete subtype is invalid.
///
/// If the sub-level of the MIME type is a wildcard, returns all extensions for the top-level.
///
/// Wildcard results contain each extension once, in lexicographic order.
///
#[cfg(feature = "rev-map")]
pub fn get_mime_extensions_str(mime_str: &str) -> Option<&'static [&'static str]> {
    if mime_str.bytes().any(|byte| byte.is_ascii_control()) {
        return None;
    }

    let mime_str = mime_str.trim_matches(' ');
    let essence = mime_str
        .split_once(';')
        .map_or(mime_str, |(essence, _)| essence)
        .trim_matches(' ');
    let (top, sub) = essence.split_once('/')?;

    get_extensions(top, sub)
}

/// Get the extensions for a given top-level and sub-level of a MIME type
/// (`{toplevel}/{sublevel}`).
///
/// Returns `None` if `toplevel` or `sublevel` are invalid or unknown.
/// Both components must be nonempty ASCII tokens without surrounding spaces.
///
/// ### Wildcards
/// `*/*` returns all extensions. A wildcard top-level with a concrete subtype is invalid.
///
/// If the sub-level of the MIME type is a wildcard, returns all extensions for the top-level.
///
/// Wildcard results contain each extension once, in lexicographic order.
#[cfg(feature = "rev-map")]
pub fn get_extensions(toplevel: &str, sublevel: &str) -> Option<&'static [&'static str]> {
    let valid = match (toplevel, sublevel) {
        ("*", "*") => true,
        ("*", _) => false,
        (_, "*") => is_mime_token(toplevel),
        _ => is_mime_token(toplevel) && is_mime_token(sublevel),
    };
    if !valid {
        return None;
    }

    impl_::get_extensions(toplevel, sublevel)
}

#[cfg(feature = "rev-map")]
fn is_mime_token(value: &str) -> bool {
    // HTTP token characters, excluding `*`, which is only accepted as a whole wildcard.
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

/// Get the MIME type for `application/octet-stream` (generic binary stream)
#[deprecated(since = "2.0.0", note = "use `mime::APPLICATION_OCTET_STREAM` instead")]
pub fn octet_stream() -> Mime {
    mime::APPLICATION_OCTET_STREAM
}

#[cfg(test)]
mod tests {
    include!("mime_types.rs");

    #[cfg(feature = "rev-map")]
    use super::get_mime_extensions_str;
    use super::{expect_mime, from_ext, from_path};
    use std::fmt::Debug;
    use std::path::Path;

    #[test]
    fn check_type_bounds() {
        fn assert_type_bounds<T: Clone + Debug + Send + Sync + 'static>() {}

        assert_type_bounds::<super::MimeInfer>();
        assert_type_bounds::<super::Iter>();
        assert_type_bounds::<super::IterRaw>();

        let empty = super::MimeInfer::default();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.count(), 0);
    }

    #[test]
    fn test_mime_type_inferring() {
        assert_eq!(
            from_ext("gif").first_or_octet_stream().to_string(),
            "image/gif".to_string()
        );
        assert_eq!(
            from_ext("TXT").first_or_octet_stream().to_string(),
            "text/plain".to_string()
        );
        assert_eq!(
            from_ext("blahblah").first_or_octet_stream().to_string(),
            "application/octet-stream".to_string()
        );
        assert_eq!(from_ext("rpm").first_raw(), Some("application/x-rpm"));
        assert_eq!(from_ext("zst").first_raw(), Some("application/zstd"));

        assert_eq!(
            from_path(Path::new("/path/to/file.gif"))
                .first_or_octet_stream()
                .to_string(),
            "image/gif".to_string()
        );
        assert_eq!(
            from_path("/path/to/file.gif")
                .first_or_octet_stream()
                .to_string(),
            "image/gif".to_string()
        );
    }

    #[test]
    fn test_mime_type_inferring_opt() {
        assert_eq!(
            from_ext("gif").first().unwrap().to_string(),
            "image/gif".to_string()
        );
        assert_eq!(
            from_ext("TXT").first().unwrap().to_string(),
            "text/plain".to_string()
        );
        assert_eq!(from_ext("blahblah").first(), None);

        assert_eq!(
            from_path("/path/to/file.gif").first().unwrap().to_string(),
            "image/gif".to_string()
        );
        assert_eq!(from_path("/path/to/file").first(), None);
    }

    #[test]
    fn test_are_mime_types_parseable() {
        for (_, mimes) in MIME_TYPES {
            mimes.iter().for_each(|s| {
                expect_mime(s);
            });
        }
    }

    // RFC: Is this test necessary anymore? --@cybergeek94, 2/1/2016
    #[test]
    fn test_are_extensions_ascii() {
        for (ext, _) in MIME_TYPES {
            assert!(ext.is_ascii(), "Extension not ASCII: {:?}", ext);
        }
    }

    #[test]
    fn test_are_extensions_sorted() {
        // simultaneously checks the requirement that duplicate extension entries are adjacent
        for (&(ext, _), &(n_ext, _)) in MIME_TYPES.iter().zip(MIME_TYPES.iter().skip(1)) {
            assert!(
                ext <= n_ext,
                "Extensions in src/mime_types should be sorted lexicographically
                in ascending order. Failed assert: {:?} <= {:?}",
                ext,
                n_ext
            );
        }
    }

    #[test]
    fn extension_length_bound_preserves_all_mappings() {
        for &(ext, expected) in MIME_TYPES {
            assert!(ext.len() <= super::impl_::MAX_EXT_INPUT_BYTES);
            assert_eq!(from_ext(ext).iter_raw().collect::<Vec<_>>(), expected);
            assert_eq!(
                from_ext(&ext.to_ascii_uppercase())
                    .iter_raw()
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }

    #[test]
    fn extension_length_bound_preserves_unicode_case_folding() {
        // Kelvin sign, long s, and the fl ligature fold to ASCII, including a
        // fold that expands a single input scalar into multiple output scalars.
        for (unicode, ascii) in [
            ("\u{212a}tx", "ktx"),
            ("\u{017f}vg", "svg"),
            ("\u{fb02}ac", "flac"),
        ] {
            assert!(!from_ext(ascii).is_empty());
            assert_eq!(from_ext(unicode), from_ext(ascii));
        }
    }

    #[test]
    fn unicode_case_folding_does_not_remove_input_scalars() {
        // The length bound relies on this property of the case-folding table.
        // Check every Unicode scalar so dependency upgrades cannot silently
        // invalidate the bound's proof.
        for scalar in (0..=0x10ffff).filter_map(char::from_u32) {
            let mut buf = [0; 4];
            let input = scalar.encode_utf8(&mut buf);
            let folded = unicase::UniCase::unicode(input).to_folded_case();
            assert!(!folded.is_empty(), "case folding removed {scalar:?}");
        }
    }

    #[test]
    fn overlong_extensions_are_unknown() {
        let bound = super::impl_::MAX_EXT_INPUT_BYTES;
        assert!(from_ext(&"z".repeat(bound + 1)).is_empty());
        assert!(from_ext(&"z".repeat(1_000_000)).is_empty());
        assert!(from_ext(&"\u{212a}".repeat(bound + 1)).is_empty());
        assert!(from_path(format!("file.{}", "z".repeat(1_000_000))).is_empty());
    }

    #[test]
    #[cfg(feature = "rev-map")]
    fn test_get_mime_extensions_str_no_panic_if_bad_mime() {
        assert_eq!(get_mime_extensions_str(""), None);
        assert_eq!(get_mime_extensions_str("not-a-mime"), None);
    }

    #[test]
    #[cfg(feature = "rev-map")]
    fn test_get_mime_extensions_str_ignores_parameters_and_whitespace() {
        assert_eq!(
            get_mime_extensions_str(" text/plain ; charset=utf-8 "),
            get_mime_extensions_str("text/plain")
        );
    }
}
