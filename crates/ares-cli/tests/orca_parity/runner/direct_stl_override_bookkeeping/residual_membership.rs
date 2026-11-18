//! Closed dynamic-only membership set from full-config-residual-membership.md.
use super::{FILAMENT, MACHINE, OrcaRunner, PROCESS, profile_entry};
use ares_core::{ConfigOverrides, Placement};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

const KEYS: &[&str] = &[
    "default_print_profile",
    "extruder_variant_list",
    "flashforge_serial_number",
    "group_algo_with_time",
    "host_type",
    "ironing_expansion",
    "pellet_modded_printer",
    "plugins",
    "printer_agent",
    "printer_plugin_config_overrides",
    "printhost_authorization_type",
    "printhost_ssl_ignore_revoke",
    "thumbnails_format",
    "upward_compatible_machine",
];

fn rows<'a>(output: &'a [u8], key: &str) -> Vec<&'a str> {
    let text = std::str::from_utf8(output).unwrap();
    let block = text.split_once("; CONFIG_BLOCK_START\n").unwrap().1;
    let block = block.split_once("; CONFIG_BLOCK_END").unwrap().0;
    let prefix = format!("; {key} = ");
    block
        .split('\n')
        .filter(|line| line.starts_with(&prefix))
        .collect()
}

fn slice(applied: Option<(&str, Value)>, label: &str) -> (PathBuf, Vec<u8>, Vec<u8>) {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = crate::runner::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "WEMAKE3D").unwrap();
    let mut vendor = profile_entry::vendor(&profiles_root, "WEMAKE3D");
    if let Some((key, value)) = &applied {
        let (kind, name) = if *key == "ironing_expansion" {
            ("process", PROCESS)
        } else {
            ("machine", MACHINE)
        };
        let profile = vendor["files"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .find(|profile| profile["type"] == kind && profile["name"] == name)
            .unwrap();
        profile[*key] = value.clone();
    }
    let catalog = json!({"vendors": [
        profile_entry::vendor(&profiles_root, "OrcaFilamentLibrary"), vendor
    ], "imports": []});
    let selection = json!({"machine": MACHINE, "process": PROCESS, "filaments": [FILAMENT]});
    let input =
        fs::read(root.join("tests/parity/counterbore-partiallybridge/solid-plate.stl")).unwrap();
    profile_entry::write_json(&runner.work.join("catalog.json"), &catalog);
    profile_entry::write_json(&runner.work.join("selection.json"), &selection);
    fs::write(runner.work.join("input.stl"), &input).unwrap();
    for (kind, mut fields) in [
        ("machine", profiles.machine(MACHINE).unwrap()),
        ("process", profiles.process(PROCESS).unwrap()),
        ("filament", profiles.filament(FILAMENT).unwrap()),
    ] {
        if let Some((key, value)) = &applied {
            let owner = if *key == "ironing_expansion" {
                "process"
            } else {
                "machine"
            };
            if kind == owner {
                // Config.cpp:984-1063: Native preset JSON stores scalar tokens as strings.
                let native_value = match value {
                    Value::Bool(value) => json!(u8::from(*value).to_string()),
                    Value::Number(value) => json!(value.to_string()),
                    _ => value.clone(),
                };
                fields.insert((*key).to_owned(), native_value);
            }
        }
        profile_entry::write_json(&runner.work.join(format!("{kind}.json")), &json!(fields));
    }
    let native_dir = runner.work.join("native");
    fs::create_dir(&native_dir).unwrap();
    let path = |name: &str| runner.work.join(name).to_str().unwrap().to_owned();
    crate::runner::stages::run(
        runner.bin(),
        [
            "--load-settings".to_owned(),
            format!("{};{}", path("machine.json"), path("process.json")),
            "--load-filaments".to_owned(),
            path("filament.json"),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            native_dir.to_str().unwrap().to_owned(),
            path("input.stl"),
        ],
        crate::runner::stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    let native = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    fs::write(runner.work.join("native.gcode"), &native).unwrap();
    let actual = crate::runner::profile_entry::slice_flattened(
        (&input, "input.stl"),
        &[
            runner.work.join("machine.json"),
            runner.work.join("process.json"),
        ],
        &[runner.work.join("filament.json")],
        &ConfigOverrides::default(),
        Placement::None,
    )
    .unwrap();
    fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    profile_entry::write_json(
        &runner.work.join("residual-membership-result.json"),
        &json!({
            "case": label, "applied": applied,
            "input_sha256": crate::runner::cache_identity::raw_digest(&input),
            "native_sha256": crate::runner::cache_identity::raw_digest(&native),
            "api_sha256": crate::runner::cache_identity::raw_digest(&actual),
            "rust_entry": "ares_core::slice_with_load_settings", "native": "fresh runtime CLI; no stored values",
        }),
    );
    (runner.work.clone(), native, actual)
}

#[test]
fn absent_dynamic_only_defaults_are_not_emitted() {
    if !crate::runner::reference_available() {
        return;
    }
    let (work, native, actual) = slice(None, "absent-fourteen");
    for key in KEYS {
        assert!(
            rows(&native, key).is_empty(),
            "Native absent premise {key}; {work:?}"
        );
        assert!(
            rows(&actual, key).is_empty(),
            "default-only membership {key}; {work:?}"
        );
    }
}

macro_rules! applied_control {
    ($test:ident, $key:literal, $default:expr, $nondefault:expr) => {
        #[test]
        fn $test() {
            if !crate::runner::reference_available() {
                return;
            }
            for (label, value) in [("default", $default), ("nondefault", $nondefault)] {
                let (work, native, actual) = slice(Some(($key, value)), label);
                let expected = rows(&native, $key);
                assert_eq!(
                    expected.len(),
                    1,
                    "Native applied premise {}; {work:?}",
                    $key
                );
                assert_eq!(
                    rows(&actual, $key),
                    expected,
                    "public applied value {}; {work:?}",
                    $key
                );
            }
        }
    };
}

applied_control!(
    applied_default_print_profile,
    "default_print_profile",
    json!(""),
    json!(PROCESS)
);
applied_control!(
    applied_extruder_variant_list,
    "extruder_variant_list",
    json!(["Direct Drive Standard"]),
    json!(["Direct Drive Standard,Direct Drive High Flow"])
);
applied_control!(
    applied_flashforge_serial_number,
    "flashforge_serial_number",
    json!(""),
    json!("ARES-MEMBERSHIP")
);
applied_control!(
    applied_group_algo_with_time,
    "group_algo_with_time",
    json!(false),
    json!(true)
);
applied_control!(
    applied_host_type,
    "host_type",
    json!("octoprint"),
    json!("prusalink")
);
applied_control!(
    applied_ironing_expansion,
    "ironing_expansion",
    json!(0.0),
    json!(1.25)
);
applied_control!(
    applied_pellet_modded_printer,
    "pellet_modded_printer",
    json!(false),
    json!(true)
);
applied_control!(
    applied_plugins,
    "plugins",
    json!([]),
    json!(["ares-membership"])
);
applied_control!(
    applied_printer_agent,
    "printer_agent",
    json!(""),
    json!("ares-membership")
);
applied_control!(
    applied_printer_plugin_config_overrides,
    "printer_plugin_config_overrides",
    json!(""),
    json!("{}")
);
applied_control!(
    applied_printhost_authorization_type,
    "printhost_authorization_type",
    json!("key"),
    json!("user")
);
applied_control!(
    applied_printhost_ssl_ignore_revoke,
    "printhost_ssl_ignore_revoke",
    json!(false),
    json!(true)
);
applied_control!(
    applied_thumbnails_format,
    "thumbnails_format",
    json!("PNG"),
    json!("JPG")
);
applied_control!(
    applied_upward_compatible_machine,
    "upward_compatible_machine",
    json!([]),
    json!([MACHINE])
);
