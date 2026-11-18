use crate::runner::{native_api_cli::sha, native_stream_case};
use serde_json::json;
use std::io::{Cursor, Read, Write};

const BASE: &[u8] = include_bytes!("../../../../../../tests/parity/semm-ramming/R1.3mf");
const SETTINGS: &str = "Metadata/project_settings.config";
const SOURCES: &[&str] = &[
    "WipeTower2.cpp:1132-1145 tower-local material flags",
    "WipeTower2.cpp:2353-2386 finish extrusion saving",
    "WipeTower2.cpp:2405-2431 finish filament selection",
    "WipeTower2.cpp:2552-2577 finish and toolchange emission",
];

fn project(flag: Option<&str>, forced: bool) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut file = source.by_index(index).unwrap();
        if file.name() != SETTINGS {
            output.raw_copy_file(file).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        let mut config: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(config["wipe_tower_type"], "type2");
        assert_eq!(config["enable_support"], "0");
        assert_eq!(config["enable_prime_tower"], "1");
        assert_eq!(config["wipe_tower_filament"], "0");
        assert_eq!(config["filament_is_support"], json!(["0", "0"]));
        assert_eq!(config["filament_soluble"], json!(["0", "0"]));
        if let Some(flag) = flag {
            config[flag] = json!(["0", "1"]);
        }
        if forced {
            config["wipe_tower_filament"] = json!("1");
        }
        output
            .start_file(SETTINGS, zip::write::SimpleFileOptions::default())
            .unwrap();
        output
            .write_all(&serde_json::to_vec(&config).unwrap())
            .unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn check(label: &str, flag: Option<&str>, forced: bool) {
    let project = project(flag, forced);
    native_stream_case::check(
        ("type2-finish-filament", SOURCES),
        label,
        &project,
        &sha(&project),
        (true, |reference, _| {
            assert!(reference.native.contains("; CP TOOLCHANGE START\n"));
            assert!(reference.native.contains(";TYPE:Prime tower\n"));
            assert!(!reference.native.contains(";TYPE:Support"));
        }),
    );
}

#[test]
fn automatic_finish_uses_the_incoming_filament_before_support() {
    check("automatic-support", Some("filament_is_support"), false);
}

#[test]
fn automatic_finish_uses_the_incoming_filament_before_soluble() {
    check("automatic-soluble", Some("filament_soluble"), false);
}

#[test]
fn automatic_finish_with_ordinary_filaments_matches_native() {
    check("automatic-control", None, false);
}

#[test]
fn forced_finish_with_ordinary_filaments_matches_native() {
    check("forced-control", None, true);
}

#[test]
fn forced_finish_with_a_support_filament_matches_native() {
    check("forced-support", Some("filament_is_support"), true);
}
