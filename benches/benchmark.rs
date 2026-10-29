use divan::{Bencher, black_box, counter::ItemsCount};
use mime_infer::{from_ext, from_path};

include!("../src/mime_types.rs");

fn main() {
    divan::main();
}

// Batch timings include one lookup per table entry. The item counter reports
// lookup throughput; the displayed time is for the entire batch.
#[divan::bench]
fn from_ext_lowercase_raw(bencher: Bencher) {
    bencher
        .counter(ItemsCount::new(MIME_TYPES.len()))
        .bench(|| {
            for (ext, _) in MIME_TYPES {
                black_box(from_ext(black_box(ext)).first_raw());
            }
        });
}

#[divan::bench]
fn from_ext_uppercase_raw(bencher: Bencher) {
    // Prepare all strings before entering the measured closure, matching the
    // lowercase benchmark's scope: lookup only, without conversion/allocation.
    let uppercased: Vec<_> = MIME_TYPES
        .iter()
        .map(|(ext, _)| ext.to_ascii_uppercase())
        .collect();

    bencher
        .counter(ItemsCount::new(uppercased.len()))
        .bench(|| {
            for ext in &uppercased {
                black_box(from_ext(black_box(ext)).first_raw());
            }
        });
}

#[divan::bench]
fn from_ext_lowercase_parsed(bencher: Bencher) {
    bencher
        .counter(ItemsCount::new(MIME_TYPES.len()))
        .bench(|| {
            for (ext, _) in MIME_TYPES {
                // Include parsing and dropping each returned Mime, as a caller of
                // first() would, to compare with the raw-string API separately.
                black_box(from_ext(black_box(ext)).first());
            }
        });
}

#[divan::bench(args = ["unknown-extension", "", ".txt"])]
fn from_ext_unknown(bencher: Bencher, ext: &str) {
    bencher.counter(ItemsCount::new(1usize)).bench(|| {
        black_box(from_ext(black_box(ext)).first_raw());
    });
}

// Let Divan batch fast length rejections above the timer's resolution. Each
// input is allocated once outside the timer and reused across iterations.
#[divan::bench(args = [256, 4096, 1_000_000], sample_count = 50)]
fn from_ext_long_unknown(bencher: Bencher, len: usize) {
    let ext = "z".repeat(len);
    bencher.counter(ItemsCount::new(1usize)).bench(|| {
        black_box(from_ext(black_box(ext.as_str())).first_raw());
    });
}

#[divan::bench(args = ["\u{212a}tx", "\u{017f}vg", "\u{fb02}ac"])]
fn from_ext_unicode(bencher: Bencher, ext: &str) {
    bencher.counter(ItemsCount::new(1usize)).bench(|| {
        black_box(from_ext(black_box(ext)).first_raw());
    });
}

#[divan::bench]
fn from_path_raw(bencher: Bencher) {
    let paths: Vec<_> = MIME_TYPES
        .iter()
        .map(|(ext, _)| format!("/assets/file.{ext}"))
        .collect();
    bencher.counter(ItemsCount::new(paths.len())).bench(|| {
        for path in &paths {
            black_box(from_path(black_box(path)).first_raw());
        }
    });
}

#[cfg(feature = "rev-map")]
#[divan::bench(args = ["text/plain", "application/x-unknown", "text/*", "*/*"])]
fn reverse_lookup(bencher: Bencher, mime: &str) {
    bencher.counter(ItemsCount::new(1usize)).bench(|| {
        black_box(mime_infer::get_mime_extensions_str(black_box(mime)));
    });
}
