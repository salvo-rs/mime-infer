# Changelog

## 4.1.0

Released on 2026-10-29. Requires Rust 1.95 or later and `mime` 0.3.17 or later.

- Add `Default` and `len` to `MimeInfer` and improve iterator behavior. Counting,
  skipping, and selecting MIME values avoid parsing discarded entries.
- Bound work for oversized extension and reverse MIME lookup inputs. Document
  Unicode case folding and the security limits of extension-based inference.
- Validate reverse MIME queries, including wildcards and parameters, and return
  sorted, deduplicated extension lists without runtime allocation.
- Correct registered MIME associations for calendar, image, document, archive,
  and Matroska formats; prefer current registrations while retaining applicable
  legacy aliases.
- Improve feature documentation and benchmark sampling. Test both lookup
  backends, with and without reverse mappings, on Windows and Linux, Rust 1.95
  and stable. Check the minimum supported `mime` dependency explicitly.
- Add dependency auditing and automated tag releases with verified crates.io
  packages and SHA-256 checksums attached to GitHub Releases.
