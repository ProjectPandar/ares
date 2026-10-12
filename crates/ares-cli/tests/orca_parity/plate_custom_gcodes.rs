//! `Model::plates_custom_gcodes` after `Model::read_from_file`
//! (`bbs_3mf.cpp:3208-3292` and the `Model.cpp:365-366` mode check) for the
//! fixtures in `tests/parity/plate-custom-gcodes/`, against the Native dump of
//! `tools/chain-probe/plate-custom-gcodes.nix`, generated at runtime.
use ares_core::{CustomGCodeInfo, CustomGCodeMode};
use serde_json::{Value, json};
use std::process::Command;

const KEY: &str = "ARES_PLATE_CUSTOM_GCODES_BIN";

enum Native {
    Loaded(Value),
    Failed(String),
}

fn fixture(name: &str) -> std::path::PathBuf {
    crate::runner::repo_root().join(format!("tests/parity/plate-custom-gcodes/{name}.3mf"))
}

fn native(name: &str) -> Option<Native> {
    let bin = crate::native_environment::binary(KEY)?;
    let work = tempfile::tempdir().unwrap();
    let dump = work.path().join("plates_custom_gcodes.json");
    let output = Command::new(bin)
        .args(["--slice", "0", "--outputdir"])
        .arg(work.path())
        .arg(fixture(name))
        .env("ARES_PLATE_CUSTOM_GCODES_DUMP", &dump)
        .output()
        .unwrap();
    Some(if output.status.success() {
        Native::Loaded(serde_json::from_slice(&std::fs::read(&dump).unwrap()).unwrap())
    } else {
        assert!(!dump.exists(), "{name}: a failed load must not dump");
        Native::Failed(String::from_utf8_lossy(&output.stderr).into_owned())
    })
}

/// The dump's shape: `CustomGCode::Mode` and `CustomGCode::Type` as their
/// upstream integers, `print_z` as f64 bits.
fn info_json(plate: i32, info: &CustomGCodeInfo) -> Value {
    let mode = match info.mode {
        CustomGCodeMode::Undef => 0,
        CustomGCodeMode::SingleExtruder => 1,
        CustomGCodeMode::MultiAsSingle => 2,
        CustomGCodeMode::MultiExtruder => 3,
    };
    let gcodes = info
        .gcodes
        .iter()
        .map(|item| {
            json!({
                "print_z": item.print_z.to_bits(),
                "type": item.kind as u8,
                "extruder": item.extruder,
                "color": item.color,
                "extra": item.extra,
            })
        })
        .collect::<Vec<_>>();
    json!({ "plate": plate, "mode": mode, "gcodes": gcodes })
}

fn native_plates(dump: &Value) -> Vec<Value> {
    dump["plates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|plate| {
            let mut plate = plate.clone();
            for item in plate["gcodes"].as_array_mut().unwrap() {
                item["print_z"] = json!(item["print_z"].as_f64().unwrap().to_bits());
            }
            plate
        })
        .collect()
}

/// Compares the loaded map and the current plate's entry; returns the Native plates.
fn assert_loaded(name: &str) -> Option<Vec<Value>> {
    let dump = match native(name)? {
        Native::Loaded(dump) => dump,
        Native::Failed(log) => panic!("{name}: Native load failed: {log}"),
    };
    let project = ares_core::load_project(std::fs::read(fixture(name)).unwrap())
        .unwrap_or_else(|error| panic!("{name}: {error}"));
    let expected = native_plates(&dump);
    let actual = project
        .plates_custom_gcodes()
        .iter()
        .map(|(plate, info)| info_json(*plate, info))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{name}: plates_custom_gcodes");

    let curr = dump["curr_plate_index"].as_i64().unwrap();
    let expected_curr = expected
        .iter()
        .find(|plate| plate["plate"].as_i64() == Some(curr))
        .cloned()
        .unwrap_or_else(|| json!({ "plate": curr, "mode": 0, "gcodes": [] }));
    assert_eq!(
        info_json(curr as i32, &project.curr_plate_custom_gcodes()),
        expected_curr,
        "{name}: get_curr_plate_custom_gcodes"
    );
    Some(expected)
}

fn kinds(plates: &[Value]) -> std::collections::BTreeSet<u64> {
    plates
        .iter()
        .flat_map(|plate| plate["gcodes"].as_array().unwrap())
        .map(|item| item["type"].as_u64().unwrap())
        .collect()
}

#[test]
fn plates_keys_skips_and_appends_like_native() {
    let Some(plates) = assert_loaded("plates") else {
        return;
    };
    assert_eq!(kinds(&plates).len(), 6, "plates: every CustomGCode::Type");
    assert!(plates.len() > 1, "plates: more than one plate");
}

#[test]
fn root_children_go_to_the_first_plate_like_native() {
    let Some(plates) = assert_loaded("no-plate") else {
        return;
    };
    assert_eq!(plates.len(), 1, "no-plate: one plate");
}

#[test]
fn legacy_gcode_attribute_like_native() {
    let Some(plates) = assert_loaded("legacy") else {
        return;
    };
    assert_eq!(
        kinds(&plates).len(),
        4,
        "legacy: ColorChange, PausePrint, ToolChange, Custom"
    );
}

#[test]
fn mode_check_like_native() {
    let Some(plates) = assert_loaded("modes") else {
        return;
    };
    let modes = plates
        .iter()
        .map(|plate| plate["mode"].as_u64().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(modes, [1, 2, 3].into(), "modes: every resolved mode");
}

#[test]
fn wrong_root_and_empty_entry_load_nothing_like_native() {
    for name in ["wrong-root", "empty"] {
        let Some(plates) = assert_loaded(name) else {
            return;
        };
        assert!(plates.is_empty(), "{name}: no plates");
    }
}

#[test]
fn missing_attribute_fails_the_load_like_native() {
    let Some(native) = native("missing-attr") else {
        return;
    };
    let Native::Failed(log) = native else {
        panic!("missing-attr: Native loaded the project");
    };
    assert!(
        log.contains("No such node (<xmlattr>.color)"),
        "missing-attr: Native log {log}"
    );
    let error = ares_core::load_project(std::fs::read(fixture("missing-attr")).unwrap())
        .expect_err("missing-attr: Ares loaded the project");
    assert!(
        matches!(error, ares_core::SliceError::InvalidInput(_)),
        "missing-attr: {error}"
    );
}
