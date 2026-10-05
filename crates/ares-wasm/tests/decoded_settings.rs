//! Native expectations for the browser settings E2E (`web/e2e/settings.spec.ts`):
//! the `loadScene` settings of the parity fixture decoded by
//! `decode_config_value_by_key` (ARD-0108), one stored element at a time as the
//! app decodes them for its CheckBox / SpinCtrl / Choice / PointCtrl fields.
//! The E2E proves the browser wiring; decoding parity with upstream
//! `ConfigOption*::deserialize` is covered by the ares-core decode tests.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const FIXTURE: &str = "tests/parity/orca_cli_ender3.3mf";
/// Option types the settings fields show as typed values.
const DECODED: [&str; 13] = [
    "bool",
    "bools",
    "bools_nullable",
    "ints",
    "ints_nullable",
    "floats",
    "floats_nullable",
    "percents",
    "percents_nullable",
    "enums",
    "enums_nullable",
    "point",
    "points",
];

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
    Other(serde::de::IgnoredAny),
}

#[derive(Deserialize)]
struct Scene {
    settings: BTreeMap<String, BTreeMap<String, Stored>>,
}

fn decode(key: &str, serialized: &str) -> String {
    ares_core::decode_config_value_by_key(key, serialized)
        .unwrap_or_else(|error| panic!("{key} = {serialized:?}: {error}"))
}

/// One stored element decodes to a one-element vector; unwrap it.
fn decode_element(key: &str, element: &str) -> String {
    let one = decode(key, element);
    one[1..one.len() - 1].to_owned()
}

#[test]
fn browser_decoded_settings_expectations() {
    let defs: Defs = serde_json::from_slice(ares_core::print_config_def_json()).unwrap();
    let decoded: Vec<String> = defs
        .options
        .into_iter()
        .filter(|def| DECODED.contains(&def.kind.as_str()))
        .map(|def| def.key)
        .collect();
    let input = fs::read(repo_root().join(FIXTURE)).unwrap();
    let scene: Scene =
        serde_json::from_str(&ares_wasm::scene_from_bytes(&input).unwrap().metadata()).unwrap();

    let mut json = String::from("{");
    let mut count = 0;
    for (at, (section, values)) in scene.settings.iter().enumerate() {
        let separator = if at == 0 { "" } else { "," };
        write!(json, "{separator}\"{section}\":{{").unwrap();
        let mut first = true;
        for (key, stored) in values.iter().filter(|(key, _)| decoded.contains(key)) {
            let value = match stored {
                Stored::Text(text) => decode(key, text),
                Stored::List(elements) => {
                    let elements: Vec<String> =
                        elements.iter().map(|e| decode_element(key, e)).collect();
                    format!("[{}]", elements.join(","))
                }
                Stored::Other(_) => panic!("{key}: unexpected stored shape"),
            };
            let separator = if first { "" } else { "," };
            write!(json, "{separator}\"{key}\":{value}").unwrap();
            first = false;
            count += 1;
        }
        json.push('}');
    }
    json.push('}');
    assert!(count > 200, "only {count} decoded values");
    let out = repo_root().join("target/web-e2e");
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("expected-decoded.json"), json).unwrap();
}
