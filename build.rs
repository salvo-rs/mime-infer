use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::{self, Path};

#[cfg(feature = "rev-map")]
use std::collections::{BTreeMap, BTreeSet};

use mime_types::MIME_TYPES;
use unicase::UniCase;

#[path = "src/mime_types.rs"]
mod mime_types;

#[cfg(feature = "phf-map")]
const PHF_PATH: &str = "phf";

#[cfg(feature = "rev-map")]
type SubLevelMap = BTreeMap<UniCase<&'static str>, Vec<&'static str>>;
#[cfg(feature = "rev-map")]
type ReverseMap = BTreeMap<UniCase<&'static str>, SubLevelMap>;

fn main() {
    println!("cargo:rerun-if-changed=src/mime_types.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let mime_types_generated_filename = "mime_types_generated.rs";
    let dest_path = Path::new(&out_dir).join(mime_types_generated_filename);
    let mut outfile = BufWriter::new(File::create(&dest_path).unwrap());

    // Unicode case folding emits at least one scalar per input scalar, and an
    // input scalar occupies at most four UTF-8 bytes. Thus no equivalent input
    // can exceed four times the longest folded key's byte length. Keep this
    // conservative bound derived from the actual table rather than assuming
    // extensions are ASCII or rejecting supported Unicode equivalents.
    let max_ext_input_bytes = MIME_TYPES
        .iter()
        .map(|(ext, _)| UniCase::new(ext).to_folded_case().len())
        .max()
        .unwrap_or(0)
        * 4;
    writeln!(
        outfile,
        "pub(crate) const MAX_EXT_INPUT_BYTES: usize = {max_ext_input_bytes};"
    )
    .unwrap();

    println!(
        "cargo:rustc-env=MIME_TYPES_GENERATED_PATH={separator}{filename}",
        separator = path::MAIN_SEPARATOR,
        filename = mime_types_generated_filename,
    );

    #[cfg(feature = "phf-map")]
    build_forward_map(&mut outfile);

    #[cfg(feature = "rev-map")]
    build_rev_map(&mut outfile);

    outfile.flush().unwrap();
}

// Build forward mappings (ext -> mime type)
#[cfg(feature = "phf-map")]
fn build_forward_map<W: Write>(out: &mut W) {
    use phf_codegen::Map as PhfMap;

    let mut forward_map = PhfMap::new();
    forward_map.phf_path(PHF_PATH);

    let mut map_entries: Vec<(&str, Vec<&str>)> = Vec::new();

    for &(key, types) in MIME_TYPES {
        if let Some(&mut (key_, ref mut values)) = map_entries.last_mut() {
            // deduplicate extensions
            if key == key_ {
                values.extend_from_slice(types);
                continue;
            }
        }

        map_entries.push((key, types.into()));
    }

    for (key, values) in map_entries {
        forward_map.entry(UniCase::new(key), format!("&{:?}", values));
    }

    writeln!(
        out,
        "static MIME_TYPES: phf::Map<UniCase<&'static str>, &'static [&'static str]> = \n{};",
        forward_map.build()
    )
    .unwrap();
}

// Build reverse mappings (mime type -> ext)
#[cfg(all(feature = "phf-map", feature = "rev-map"))]
fn build_rev_map<W: Write>(out: &mut W) {
    use phf_codegen::Map as PhfMap;

    let dyn_map = get_rev_map();

    let mut rev_map = PhfMap::new();
    rev_map.phf_path(PHF_PATH);

    let mut exts = Vec::new();

    for (top, subs) in dyn_map {
        let top_exts = unique_extensions(subs.values().flatten().copied());

        let mut sub_map = PhfMap::new();
        sub_map.phf_path(PHF_PATH);

        for (sub, sub_exts) in subs {
            let sub_start = exts.len();
            exts.extend(sub_exts);
            let sub_end = exts.len();

            sub_map.entry(sub, format!("({}, {})", sub_start, sub_end));
        }

        rev_map.entry(
            top,
            format!(
                "TopLevelExts {{ extensions: &{:?}, subs: {} }}",
                top_exts,
                sub_map.build()
            ),
        );
    }

    writeln!(
        out,
        "static REV_MAPPINGS: phf::Map<UniCase<&'static str>, TopLevelExts> = \n{};",
        rev_map.build()
    )
    .unwrap();

    writeln!(out, "const EXTS: &[&str] = &{:?};", exts).unwrap();
    write_all_extensions(out);
}

#[cfg(all(not(feature = "phf-map"), feature = "rev-map"))]
fn build_rev_map<W: Write>(out: &mut W) {
    use std::fmt::Write as _;

    macro_rules! unicase_const {
        ($s:expr) => {{
            format!(
                "{}({:?})",
                if $s.is_ascii() {
                    "UniCase::ascii"
                } else {
                    "UniCase::unicode"
                },
                $s
            )
        }};
    }

    let dyn_map = get_rev_map();

    write!(
        out,
        "static REV_MAPPINGS: &[(UniCase<&'static str>, TopLevelExts)] = &["
    )
    .unwrap();

    let mut exts = Vec::new();

    for (top, subs) in dyn_map {
        let top_exts = unique_extensions(subs.values().flatten().copied());

        let mut sub_map = String::new();

        for (sub, sub_exts) in subs {
            let sub_start = exts.len();
            exts.extend(sub_exts);
            let sub_end = exts.len();

            write!(
                sub_map,
                "({}, ({}, {})),",
                unicase_const!(sub),
                sub_start,
                sub_end
            )
            .unwrap();
        }

        write!(
            out,
            "({}, TopLevelExts {{ extensions: &{:?}, subs: &[{}] }}),",
            unicase_const!(top),
            top_exts,
            sub_map
        )
        .unwrap();
    }

    writeln!(out, "];").unwrap();

    writeln!(out, "const EXTS: &[&str] = &{:?};", exts).unwrap();
    write_all_extensions(out);
}

#[cfg(feature = "rev-map")]
fn unique_extensions<'a>(extensions: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    extensions
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(feature = "rev-map")]
fn write_all_extensions<W: Write>(out: &mut W) {
    let all_exts = unique_extensions(MIME_TYPES.iter().map(|&(ext, _)| ext));
    writeln!(out, "const ALL_EXTS: &[&str] = &{:?};", all_exts).unwrap();
}

#[cfg(feature = "rev-map")]
fn get_rev_map() -> ReverseMap {
    // First, collect all the mime type -> ext mappings)
    let mut dyn_map = BTreeMap::new();
    for &(key, types) in MIME_TYPES {
        for val in types {
            let (top, sub) = split_mime(val);
            dyn_map
                .entry(UniCase::new(top))
                .or_insert_with(BTreeMap::new)
                .entry(UniCase::new(sub))
                .or_insert_with(Vec::new)
                .push(key);
        }
    }
    dyn_map
}

#[cfg(feature = "rev-map")]
fn split_mime(mime: &str) -> (&str, &str) {
    mime.split_once('/')
        .expect("MIME type mapping must contain a slash")
}
