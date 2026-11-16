//! `encode_config_value_by_key` is the inverse of `decode_config_value_by_key`,
//! byte for byte, on the `; key = value` lines Native OrcaSlicer writes between
//! `; CONFIG_BLOCK_START` and `; CONFIG_BLOCK_END` for `tests/parity/orca_cli_ender3.3mf`:
//! whole `ConfigOption::serialize()` texts, string vectors included
//! (GCode.cpp:5631-5641 `append_full_config`). The extra fixed-precision
//! `wipe_tower_x`/`wipe_tower_y` line GCode.cpp:5634-5636 writes before the
//! `opt_serialize` one is not `serialize()`; the last line per key is kept.
//!
//! Writes `config-block-cases.json` (replayed in the browser by
//! `web/e2e/configRoundTrip.spec.ts`) and `config-block-roundtrip.json` into
//! the runner work directory. The 3MF half is `crates/ares-wasm/tests/encode_roundtrip.rs`.
#[path = "config_block_roundtrip/complete_block.rs"]
mod complete_block;
#[path = "config_block_roundtrip/definition_defaults.rs"]
mod definition_defaults;
#[path = "config_block_roundtrip/nozzle_volume_types.rs"]
mod nozzle_volume_types;
#[path = "config_block_roundtrip/nullable_vectors.rs"]
mod nullable_vectors;
#[path = "config_block_roundtrip/preset_include_controls.rs"]
mod preset_include_controls;
#[path = "config_block_roundtrip/preset_include_e3d.rs"]
mod preset_include_e3d;
#[path = "config_block_roundtrip/project_catalog_cli.rs"]
mod project_catalog_cli;
#[path = "config_block_roundtrip/project_catalog_cli_errors.rs"]
mod project_catalog_cli_errors;
#[path = "config_block_roundtrip/project_catalog_cli_protocol.rs"]
mod project_catalog_cli_protocol;
#[path = "config_block_roundtrip/project_catalog_supply.rs"]
mod project_catalog_supply;
#[path = "config_block_roundtrip/project_filament_miss.rs"]
mod project_filament_miss;
#[path = "config_block_roundtrip/project_preset_refresh.rs"]
mod project_preset_refresh;
#[path = "config_block_roundtrip/project_printer_miss.rs"]
mod project_printer_miss;
#[path = "config_block_roundtrip/project_process_miss.rs"]
mod project_process_miss;
#[path = "config_block_roundtrip/project_refresh_errors.rs"]
mod project_refresh_errors;

use super::{ExportedCase, OrcaRunner};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = "tests/parity/orca_cli_ender3.3mf";

#[derive(Deserialize)]
struct Defs {
    options: Vec<Def>,
}

#[derive(Deserialize)]
struct Def {
    key: String,
}

/// The shape of `encode_roundtrip.rs` cases, so the browser replays both alike.
#[derive(Serialize)]
struct Case<'a> {
    source: &'static str,
    key: &'a str,
    element: Option<usize>,
    original: &'a str,
}

#[derive(Serialize)]
struct Failure<'a> {
    #[serde(flatten)]
    case: &'a Case<'a>,
    decoded: Option<String>,
    encoded: Result<String, String>,
}

#[test]
fn config_block_encode_inverts_decode() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let project = std::fs::read(super::repo_root().join(FIXTURE)).unwrap();
    let project_path = runner.work.join("orca_cli_ender3.3mf");
    let output_dir = runner.work.join("config-block-native");
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    let sliced = runner
        .slice_case(&ExportedCase {
            label: "orca_cli_ender3".to_owned(),
            project,
            project_path,
            output_dir,
        })
        .unwrap_or_else(|error| panic!("{error}"));
    let gcode = std::str::from_utf8(&sliced.reference).unwrap();
    let defs: Defs = serde_json::from_slice(ares_core::print_config_def_json()).unwrap();
    let defs: BTreeSet<String> = defs.options.into_iter().map(|def| def.key).collect();
    let block = gcode.split_once("; CONFIG_BLOCK_START\n").unwrap().1;
    let block = block.split_once("; CONFIG_BLOCK_END").unwrap().0;
    let last: BTreeMap<&str, &str> = block
        .lines()
        .filter_map(|line| line.strip_prefix("; ")?.split_once(" = "))
        .filter(|(key, _)| defs.contains(*key))
        .collect();
    let cases: Vec<Case> = last
        .into_iter()
        .map(|(key, original)| Case {
            source: "orca_config_block",
            key,
            element: None,
            original,
        })
        .collect();
    let failures: Vec<Failure> = cases
        .iter()
        .filter_map(|case| {
            let (decoded, encoded) =
                match ares_core::decode_config_value_by_key(case.key, case.original) {
                    Ok(decoded) => {
                        let encoded = ares_core::encode_config_value_by_key(case.key, &decoded);
                        (Some(decoded), encoded)
                    }
                    Err(error) => (None, Err(error)),
                };
            (encoded.as_deref() != Ok(case.original)).then_some(Failure {
                case,
                decoded,
                encoded,
            })
        })
        .collect();
    std::fs::write(
        runner.work.join("config-block-cases.json"),
        serde_json::to_string(&cases).unwrap(),
    )
    .unwrap();
    let report = runner.work.join("config-block-roundtrip.json");
    std::fs::write(
        &report,
        serde_json::to_string_pretty(&serde_json::json!({
            "fixture": FIXTURE,
            "values": cases.len(),
            "failed": failures.len(),
            "failures": failures,
        }))
        .unwrap(),
    )
    .unwrap();
    // 618 PrintConfigDef keys in the 2.4.2 CONFIG_BLOCK of this fixture.
    assert!(cases.len() >= 618, "only {} values", cases.len());
    assert!(failures.is_empty(), "see {report:?}");
}
