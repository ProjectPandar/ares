//! `encode_config_value_by_key` is the inverse of `decode_config_value_by_key`
//! on real values, byte for byte, for every `PrintConfigDef` key they hold:
//!
//! - `orca_config_block`: the `; key = value` lines official OrcaSlicer 2.4.2
//!   wrote for the parity fixture (`tests/parity/replay/ender3.orca.gcode`,
//!   provenance in its README) — whole `ConfigOption::serialize()` texts,
//!   string vectors included (GCode.cpp:5631-5641 `append_full_config`). The
//!   extra fixed-precision `wipe_tower_x`/`wipe_tower_y` line GCode.cpp:5634-5636
//!   writes before the `opt_serialize` one is not `serialize()`; the last line
//!   per key is kept.
//! - `3mf:<section>`: the fixture's `loadScene` settings. Scalars are stored
//!   as `serialize()`, vectors per element as `vserialize()` (Config.cpp:1464-1495
//!   `save_to_json`); a non-string element round-trips as a one-element vector.
//!   `coString` scalars (Config.cpp:1476-1478) and string vector elements are
//!   stored raw, not as `serialize()` text, so they are counted as skipped; the
//!   CONFIG_BLOCK source covers them whole.
//!
//! Writes `target/web-e2e/encode-roundtrip.json` (totals and failures) and the
//! cases the browser round trip (`web/e2e/package.spec.ts`) replays.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const FIXTURE: &str = "tests/parity/orca_cli_ender3.3mf";
const ORCA_GCODE: &str = "tests/parity/replay/ender3.orca.gcode";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[derive(Deserialize)]
struct Defs {
    options: Vec<Def>,
}

#[derive(Deserialize)]
struct Def {
    key: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Stored {
    Text(String),
    List(Vec<String>),
}

#[derive(Deserialize)]
struct Scene {
    settings: BTreeMap<String, BTreeMap<String, Stored>>,
}

/// One stored value: a scalar, or one element of a vector.
#[derive(Serialize)]
struct Case {
    source: String,
    key: String,
    element: Option<usize>,
    original: String,
}

#[derive(Serialize)]
struct Failure {
    #[serde(flatten)]
    case: Case,
    decoded: Option<String>,
    encoded: Result<String, String>,
}

/// Per `PrintConfigDef` type: how many keys exist and which hold no real value.
#[derive(Default, Serialize)]
struct Coverage {
    keys: usize,
    uncovered: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    fixture: &'static str,
    orca_gcode: &'static str,
    def_keys: usize,
    keys: usize,
    values: usize,
    skipped_raw_strings: usize,
    by_type: BTreeMap<String, Coverage>,
    failed: usize,
    failures: Vec<Failure>,
}

/// Decodes and re-encodes `case.original`; a vector element travels as `[element]`.
fn round_trip(case: &Case) -> (Option<String>, Result<String, String>) {
    let decoded = match ares_core::decode_config_value_by_key(&case.key, &case.original) {
        Ok(decoded) => decoded,
        Err(error) => return (None, Err(error)),
    };
    let encoded = ares_core::encode_config_value_by_key(&case.key, &decoded);
    (Some(decoded), encoded)
}

/// `; key = value` lines between `; CONFIG_BLOCK_START` and `; CONFIG_BLOCK_END`.
fn config_block(gcode: &str, defs: &BTreeMap<String, String>) -> Vec<Case> {
    let block = gcode.split_once("; CONFIG_BLOCK_START\n").unwrap().1;
    let block = block.split_once("; CONFIG_BLOCK_END").unwrap().0;
    let last: BTreeMap<&str, &str> = block
        .lines()
        .filter_map(|line| line.strip_prefix("; ")?.split_once(" = "))
        .filter(|(key, _)| defs.contains_key(*key))
        .collect();
    last.into_iter()
        .map(|(key, value)| Case {
            source: "orca_config_block".to_owned(),
            key: key.to_owned(),
            element: None,
            original: value.to_owned(),
        })
        .collect()
}

/// Stored 3MF values, and the number of raw string values skipped.
fn stored(defs: &BTreeMap<String, String>, scene: Scene) -> (Vec<Case>, usize) {
    let mut cases = Vec::new();
    let mut skipped = 0;
    for (section, values) in scene.settings {
        for (key, stored) in values {
            let Some(kind) = defs.get(&key) else { continue };
            let case = |element, original| Case {
                source: format!("3mf:{section}"),
                key: key.clone(),
                element,
                original,
            };
            match stored {
                Stored::Text(_) if kind == "string" => skipped += 1,
                Stored::Text(text) => cases.push(case(None, text)),
                Stored::List(elements) if kind == "strings" => skipped += elements.len(),
                Stored::List(elements) => cases.extend(
                    elements
                        .into_iter()
                        .enumerate()
                        .map(|(at, element)| case(Some(at), element)),
                ),
            }
        }
    }
    (cases, skipped)
}

#[test]
fn encode_inverts_decode_on_real_values() {
    let defs: Defs = serde_json::from_slice(ares_core::print_config_def_json()).unwrap();
    let defs: BTreeMap<String, String> = defs
        .options
        .into_iter()
        .map(|def| (def.key, def.kind))
        .collect();
    let gcode = fs::read_to_string(repo_root().join(ORCA_GCODE)).unwrap();
    let input = fs::read(repo_root().join(FIXTURE)).unwrap();
    let scene: Scene =
        serde_json::from_str(&ares_wasm::scene_from_bytes(&input).unwrap().metadata()).unwrap();
    let mut cases = config_block(&gcode, &defs);
    let (stored, skipped_raw_strings) = stored(&defs, scene);
    cases.extend(stored);
    let covered: BTreeSet<&str> = cases.iter().map(|case| case.key.as_str()).collect();
    let keys = covered.len();
    let mut by_type = BTreeMap::<String, Coverage>::new();
    for (key, kind) in &defs {
        let coverage = by_type.entry(kind.clone()).or_default();
        coverage.keys += 1;
        if !covered.contains(key.as_str()) {
            coverage.uncovered.push(key.clone());
        }
    }

    let out = repo_root().join("target/web-e2e");
    fs::create_dir_all(&out).unwrap();
    fs::write(
        out.join("encode-roundtrip-cases.json"),
        serde_json::to_string(&cases).unwrap(),
    )
    .unwrap();
    let values = cases.len();
    let failures: Vec<Failure> = cases
        .into_iter()
        .filter_map(|case| {
            let (decoded, encoded) = round_trip(&case);
            (encoded.as_ref() != Ok(&case.original)).then_some(Failure {
                case,
                decoded,
                encoded,
            })
        })
        .collect();
    let report = Report {
        fixture: FIXTURE,
        orca_gcode: ORCA_GCODE,
        def_keys: defs.len(),
        keys,
        values,
        skipped_raw_strings,
        by_type,
        failed: failures.len(),
        failures,
    };
    fs::write(
        out.join("encode-roundtrip.json"),
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
    assert!(values > 1000, "only {values} values");
    assert_eq!(report.failed, 0, "see target/web-e2e/encode-roundtrip.json");
}
