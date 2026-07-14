use mime_infer::from_ext;

include!("../src/mime_types.rs");

fn main() {
    divan::main();
}

#[divan::bench]
fn from_ext_benchmark() {
    for (mime_ext, _) in MIME_TYPES {
        divan::black_box(from_ext(mime_ext).first_raw());
    }
}

#[divan::bench]
fn from_ext_uppercase_benchmark() {
    let uppercased = MIME_TYPES.iter().map(|(s, _)| s.to_uppercase());

    for mime_ext in uppercased {
        divan::black_box(from_ext(&mime_ext).first_raw());
    }
}
