//! Config.cpp's JSON stream and nlohmann lexer accept one UTF-8 BOM at byte zero.
//! Exercise existing public profile, catalog, import and original-machine entries.
use std::io::{Cursor, Write};

use ares_core::{
    LoadSettingsFile, PresetCatalogHandle, ProfileFragment, ProfileKind, UpwardMachineInput,
    UpwardMachineSource, filter_compatible_presets, parse_preset_bundle,
};
use serde_json::{Value, json};
use zip::{ZipWriter, write::SimpleFileOptions};

const BOM: &[u8] = b"\xef\xbb\xbf";
const ORIGINALS: [&[u8]; 5] = [
    include_bytes!("../../../tests/parity/profile-json-bom/fdm_process_benfab_04020.json"),
    include_bytes!("../../../tests/parity/profile-json-bom/fdm_process_benfab_06030.json"),
    include_bytes!("../../../tests/parity/profile-json-bom/fdm_process_benfab_08040.json"),
    include_bytes!("../../../tests/parity/profile-json-bom/fdm_process_benfab_10050.json"),
    include_bytes!("../../../tests/parity/profile-json-bom/fdm_process_benfab_common.json"),
];
const MACHINE: &[u8] = br#"{"type":"machine","name":"Bambu Lab H2D 0.4 nozzle","from":"system","printer_settings_id":"Bambu Lab H2D 0.4 nozzle","upward_compatible_machine":["one","one","two"]}"#;
const PROJECT: &[u8] =
    include_bytes!("../../../tests/parity/master-extruder-id/stock-right-only.3mf");

fn prefixed(prefix: &[u8], input: &[u8]) -> Vec<u8> {
    [prefix, input].concat()
}

fn vendor() -> Value {
    json!({
        "id": "Fixture", "index": {
            "name": "Fixture", "machine_list": [{"sub_path": "machine.json"}]
        }, "files": {"machine.json": {
            "type": "machine", "name": "Fixture", "instantiation": "true",
            "printer_model": "Fixture", "printer_notes": "\u{feff}keep"
        }}
    })
}

#[test]
fn original_benfab_bytes_preserve_metadata_payload_and_inheritance() {
    for input in ORIGINALS {
        assert_eq!(&input[..3], BOM);
        let control = ProfileFragment::from_json_bytes(&input[3..]).unwrap();
        let actual = ProfileFragment::from_json_bytes(input).unwrap();
        assert_eq!(actual.kind(), ProfileKind::Process);
        assert_eq!(actual.from(), Some("system"));
        assert_eq!(actual.instantiation(), Some("false"));
        assert_eq!(actual, control);
    }
}

#[test]
fn process_filament_and_machine_readers_preserve_string_bom_and_typed_values() {
    for input in [
        br#"{"type":"process","name":"p","layer_height":"0.27","description":"\uFEFFkeep"}"#.as_slice(),
        br#"{"type":"filament","name":"f","filament_diameter":["1.75"],"inherits":"parent","description":"\uFEFFkeep"}"#,
        br#"{"type":"machine","name":"m","nozzle_diameter":["0.6"],"description":"\uFEFFkeep"}"#,
    ] {
        let control = ProfileFragment::from_json_bytes(input).unwrap();
        let bytes = prefixed(BOM, input);
        let actual = ProfileFragment::from_json_bytes(&bytes).unwrap();
        assert_eq!(actual.description(), Some("\u{feff}keep"));
        assert_eq!(actual, control);
        assert_eq!(&bytes[..3], BOM);
        assert_eq!(&bytes[3..], input);
    }
}

#[test]
fn metadata_errors_still_precede_payload_errors() {
    for input in [
        br#"{"name":"p","layer_height":{"bad":"invalid"}}"#.as_slice(),
        br#"{"type":"unsupported","name":"p","layer_height":{"bad":"invalid"}}"#,
        br#"{"type":"process","name":"p","layer_height":{"bad":"invalid"}}"#,
    ] {
        let control = ProfileFragment::from_json_bytes(input)
            .unwrap_err()
            .to_string();
        let actual = ProfileFragment::from_json_bytes(prefixed(BOM, input))
            .unwrap_err()
            .to_string();
        assert_eq!(actual, control);
    }
}

#[test]
fn vendor_and_one_shot_catalog_bytes_have_the_same_result() {
    let vendor = vendor();
    let bytes = prefixed(BOM, vendor.to_string().as_bytes());
    let mut handle = PresetCatalogHandle::new();
    handle.add_vendor(&bytes).unwrap();
    assert_eq!(handle.printers()[0].name, "Fixture");
    let actual = serde_json::to_value(handle.filter_compatible("Fixture", None).unwrap()).unwrap();
    let catalog = json!({"vendors": [vendor], "imports": []}).to_string();
    let control =
        serde_json::to_value(filter_compatible_presets(&catalog, "Fixture", None).unwrap())
            .unwrap();
    let catalog_bom = String::from_utf8(prefixed(BOM, catalog.as_bytes())).unwrap();
    let via_catalog =
        serde_json::to_value(filter_compatible_presets(&catalog_bom, "Fixture", None).unwrap())
            .unwrap();
    assert_eq!(actual, control);
    assert_eq!(via_catalog, control);
}

#[test]
fn original_benfab_imports_keep_existing_version_admission() {
    for input in ORIGINALS {
        let control =
            serde_json::to_value(parse_preset_bundle(&input[3..], "process.json").unwrap())
                .unwrap();
        let actual =
            serde_json::to_value(parse_preset_bundle(input, "process.json").unwrap()).unwrap();
        assert_eq!(actual, control);
    }
}

#[test]
fn json_import_and_catalog_import_preserve_record_values() {
    let input = br#"{"type":"machine","name":"Fixture","version":"2.5.0","printer_settings_id":"Fixture","printer_model":"Fixture","description":"\uFEFFkeep","inherits":""}"#;
    let bytes = prefixed(BOM, input);
    let control =
        serde_json::to_value(parse_preset_bundle(input, "machine.json").unwrap()).unwrap();
    let actual =
        serde_json::to_value(parse_preset_bundle(&bytes, "machine.json").unwrap()).unwrap();
    assert_eq!(actual, control);
    assert_eq!(actual["profiles"][0]["description"], "\u{feff}keep");
    let mut handle = PresetCatalogHandle::new();
    handle.add_import(&bytes, "machine.json").unwrap();
    assert_eq!(handle.printers()[0].name, "Fixture");
}

fn archive(metadata: &[u8], profile: &[u8]) -> Vec<u8> {
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in [
        ("bundle_structure.json", metadata),
        ("machine.json", profile),
    ] {
        archive
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        archive.write_all(bytes).unwrap();
    }
    archive.finish().unwrap().into_inner()
}

#[test]
fn archive_metadata_and_json_members_accept_one_bom_without_changing_the_archive() {
    let metadata = br#"{"bundle_id":"fixture","bundle_type":"printer","author":"\uFEFFkeep","imported_time":7,"updated_time":11}"#;
    let profile = br#"{"type":"machine","name":"Fixture","version":"2.5.0","printer_settings_id":"Fixture","inherits":"parent"}"#;
    let plain = archive(metadata, profile);
    let bytes = archive(&prefixed(BOM, metadata), &prefixed(BOM, profile));
    let original = bytes.clone();
    let control =
        serde_json::to_value(parse_preset_bundle(&plain, "fixture.orca_printer").unwrap()).unwrap();
    let actual =
        serde_json::to_value(parse_preset_bundle(&bytes, "fixture.orca_printer").unwrap()).unwrap();
    assert_eq!(actual, control);
    assert_eq!(actual["bundle"]["name"], "fixture");
    assert_eq!(actual["bundle"]["description"], "printer");
    assert_eq!(actual["bundle"]["author"], "\u{feff}keep");
    assert_eq!(actual["profiles"][0]["inherits"], "parent");
    assert_eq!(bytes, original);
}

#[test]
fn original_and_up_to_date_machine_bytes_use_one_json_lexical_policy() {
    let bytes = prefixed(BOM, MACHINE);
    for input in [
        UpwardMachineInput::FetchOriginal(Some(bytes.as_slice())),
        UpwardMachineInput::UpToDateOriginal(Some(bytes.as_slice())),
    ] {
        let source = UpwardMachineSource::from_project(PROJECT).unwrap().unwrap();
        assert_eq!(source.select(input).unwrap(), ["one", "one", "two"]);
    }
    let json = std::str::from_utf8(&bytes).unwrap();
    let files = [LoadSettingsFile {
        label: "machine.json",
        json,
    }];
    let source = UpwardMachineSource::from_project(PROJECT).unwrap().unwrap();
    assert_eq!(
        source
            .select(UpwardMachineInput::UpToDateFiles(&files))
            .unwrap(),
        ["one", "one", "two"]
    );
}

#[test]
fn malformed_repeated_displaced_and_unsupported_boms_are_rejected() {
    let vendor = vendor().to_string();
    for prefix in [
        b"\xef".as_slice(),
        b"\xef\xbb",
        b"\xef\xba\xbf",
        b"\xef\xbb\xbf\xef\xbb\xbf",
        b" \xef\xbb\xbf",
        b"\xff\xfe",
        b"\xfe\xff",
    ] {
        assert!(ProfileFragment::from_json_bytes(prefixed(prefix, MACHINE)).is_err());
        assert!(parse_preset_bundle(prefixed(prefix, MACHINE), "machine.json").is_err());
        assert!(
            PresetCatalogHandle::new()
                .add_vendor(&prefixed(prefix, vendor.as_bytes()))
                .is_err()
        );
        let bytes = prefixed(prefix, MACHINE);
        let source = UpwardMachineSource::from_project(PROJECT).unwrap().unwrap();
        assert!(
            source
                .select(UpwardMachineInput::FetchOriginal(Some(&bytes)))
                .is_err()
        );
    }
    for prefix in [b"\xef\xbb\xbf\xef\xbb\xbf".as_slice(), b" \xef\xbb\xbf"] {
        let json = String::from_utf8(prefixed(prefix, MACHINE)).unwrap();
        let files = [LoadSettingsFile {
            label: "machine.json",
            json: &json,
        }];
        let source = UpwardMachineSource::from_project(PROJECT).unwrap().unwrap();
        assert!(
            source
                .select(UpwardMachineInput::UpToDateFiles(&files))
                .is_err()
        );
        let catalog = json!({"vendors": [self::vendor()], "imports": []}).to_string();
        let catalog = String::from_utf8(prefixed(prefix, catalog.as_bytes())).unwrap();
        assert!(filter_compatible_presets(&catalog, "Fixture", None).is_err());
    }
}
