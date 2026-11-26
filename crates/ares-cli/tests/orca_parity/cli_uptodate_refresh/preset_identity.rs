//! Retained machine and process identity parity (`OrcaSlicer.cpp:1851–1896,2539–2690,3098–3406`).
use std::{fs, path::Path};

use ares_core::decode_config_value_by_key;
use serde_json::{Value, json};

use super::{
    FILAMENT, KEYS, MACHINE, OrcaRunner, PROCESS, ares, artifact_dir, banner_normalized,
    native_gcode, project, settings, write_project,
};

const USER_MACHINE: &str = "Retained Custom MK3S";
const USER_PROCESS: &str = "Retained Custom Process";
const USER_FILAMENT: &str = "Retained Custom PLA";
const ALL_FILES: &[&str] = &[
    "--uptodate",
    "--uptodate-settings",
    "machine.json;process.json",
    "--uptodate-filaments",
    "filament.json",
];

#[test]
fn uptodate_flat_files_adopt_machine_and_process_source_ids() {
    with_distinct_project(|runner, root| {
        let native = compare_pair(runner, root, "explicit", ALL_FILES, &[]);
        assert_identities(&native, (MACHINE, PROCESS, USER_FILAMENT));
        assert_inherits(&native);
        assert_values(&native, ["45", "8", "1.35"]);

        // Installed `profiles/BBL/{machine,process,filament}_full` files take the same branch.
        for (kind, name) in [
            ("machine", MACHINE),
            ("process", PROCESS),
            ("filament", FILAMENT),
        ] {
            let dir = root.join(format!("resources/profiles/BBL/{kind}_full"));
            fs::create_dir_all(&dir).unwrap();
            fs::copy(
                root.join(format!("{kind}.json")),
                dir.join(format!("{name}.json")),
            )
            .unwrap();
        }
        let output = ares(
            (root, "ares-resource"),
            &["--uptodate", "--resources-dir", "resources"],
        );
        assert!(
            output.status.success(),
            "resource refresh: {}; {root:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        let actual = fs::read(root.join("ares-resource/plate_1.gcode")).unwrap();
        assert_same(&native, &actual, &root.join("ares-resource"));
    });
}

#[test]
fn uptodate_partial_settings_list_updates_only_supplied_kind_identity() {
    with_distinct_project(|runner, root| {
        // Keep empty `post_process` when stock Prusa process catalog runs under `--uptodate`.
        protect_empty_post_process(root);
        let resources = stock_resources();
        let res_arg = ["--resources-dir", resources.to_str().unwrap()];

        // Machine-only flat file: machine adopts source ID, process falls back to catalog.
        let machine_only = compare_pair(
            runner,
            root,
            "machine-only",
            &[
                "--uptodate",
                "--uptodate-settings",
                "machine.json",
                "--uptodate-filaments",
                "filament.json",
            ],
            &res_arg,
        );
        assert_identities(&machine_only, (MACHINE, USER_PROCESS, USER_FILAMENT));
        assert_inherits(&machine_only);
        assert_eq!(config(&machine_only, KEYS[0]), "45");

        // Process-only flat file: process adopts source ID, machine falls back to catalog.
        let process_only = compare_pair(
            runner,
            root,
            "process-only",
            &[
                "--uptodate",
                "--uptodate-settings",
                "process.json",
                "--uptodate-filaments",
                "filament.json",
            ],
            &res_arg,
        );
        assert_identities(&process_only, (USER_MACHINE, PROCESS, USER_FILAMENT));
        assert_inherits(&process_only);
        assert_eq!(config(&process_only, KEYS[1]), "8");
    });
}

#[test]
fn non_uptodate_controls_preserve_distinct_saved_preset_ids() {
    with_distinct_project(|runner, root| {
        let resources = stock_resources();
        let res_arg = ["--resources-dir", resources.to_str().unwrap()];

        let bare = compare_pair(runner, root, "bare", &[], &res_arg);
        assert_identities(&bare, (USER_MACHINE, USER_PROCESS, USER_FILAMENT));
        assert_inherits(&bare);
        assert_values(&bare, ["44", "6", "1.25"]);

        let inactive = compare_pair(
            runner,
            root,
            "inactive-lists",
            &[
                "--uptodate-settings",
                "machine.json;process.json",
                "--uptodate-filaments",
                "filament.json",
            ],
            &res_arg,
        );
        assert_same(&bare, &inactive, root);

        // Non-empty `different_settings_to_system` activates `refresh_unlisted` without `--uptodate`.
        protect_empty_post_process(root);
        let unlisted = compare_pair(runner, root, "non-uptodate-refresh-unlisted", &[], &res_arg);
        assert_identities(&unlisted, (USER_MACHINE, USER_PROCESS, USER_FILAMENT));
        assert_inherits(&unlisted);
        assert_ne!(config(&unlisted, KEYS[0]), "44");
        assert_ne!(config(&unlisted, KEYS[1]), "6");
    });
}

#[test]
fn uptodate_without_flat_files_preserves_distinct_saved_preset_ids() {
    with_distinct_project(|runner, root| {
        protect_empty_post_process(root);
        let resources = stock_resources();
        let res_arg = ["--resources-dir", resources.to_str().unwrap()];
        let gcode = compare_pair(runner, root, "catalog-only", &["--uptodate"], &res_arg);
        assert_identities(&gcode, (USER_MACHINE, USER_PROCESS, USER_FILAMENT));
        assert_inherits(&gcode);
        assert_ne!(config(&gcode, KEYS[0]), "44");
        assert_ne!(config(&gcode, KEYS[1]), "6");
    });
}

#[test]
fn authored_identity_overrides_keep_saved_preset_ids_during_flat_refresh() {
    with_distinct_project(|runner, root| {
        edit_project(root, |saved| {
            saved["different_settings_to_system"] =
                json!(["print_settings_id", "", "printer_settings_id"]);
        });
        let gcode = compare_pair(runner, root, "protected-ids", ALL_FILES, &[]);
        assert_identities(&gcode, (USER_MACHINE, USER_PROCESS, USER_FILAMENT));
        assert_inherits(&gcode);
        assert_values(&gcode, ["45", "8", "1.35"]);
    });
}

fn with_distinct_project(body: impl FnOnce(&OrcaRunner, &Path)) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    project(&root);
    edit_project(&root, |saved| {
        saved["printer_settings_id"] = json!(USER_MACHINE);
        saved["print_settings_id"] = json!(USER_PROCESS);
        saved["filament_settings_id"] = json!([USER_FILAMENT]);
        saved["inherits_group"] = json!([PROCESS, FILAMENT, MACHINE]);
    });
    body(&runner, &root);
}

fn protect_empty_post_process(root: &Path) {
    edit_project(root, |saved| {
        saved["post_process"] = json!([]);
        saved["different_settings_to_system"] = json!(["post_process", "", ""]);
    });
}

fn edit_project(root: &Path, edit: impl FnOnce(&mut serde_json::Map<String, Value>)) {
    let input = fs::read(root.join("retained.3mf")).unwrap();
    let mut saved = settings(&input).unwrap();
    edit(&mut saved);
    write_project(root, input, &saved);
}

fn stock_resources() -> std::path::PathBuf {
    crate::runner::repo_root().join("OrcaSlicer/resources")
}

fn compare_pair(
    runner: &OrcaRunner,
    root: &Path,
    label: &str,
    argv: &[&str],
    ares_only: &[&str],
) -> Vec<u8> {
    let native = native_gcode((runner, root), &format!("native-{label}"), argv);
    let dir = format!("ares-{label}");
    let full_argv = [argv, ares_only].concat();
    let output = ares((root, &dir), &full_argv);
    let out = root.join(&dir);
    fs::write(out.join("stdout"), &output.stdout).unwrap();
    fs::write(out.join("status"), output.status.to_string()).unwrap();
    let gcode = out.join("plate_1.gcode");
    let command_line = [
        &[env!("CARGO_BIN_EXE_ares"), "slice"][..],
        &full_argv,
        &["-o", gcode.to_str().unwrap(), "retained.3mf"],
    ]
    .concat();
    fs::write(
        out.join("argv.json"),
        serde_json::to_vec(&command_line).unwrap(),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "Ares {label}: {}; {root:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_same(&native, &fs::read(gcode).unwrap(), &out);
    native
}

fn assert_same(left: &[u8], right: &[u8], out_dir: &Path) {
    assert!(
        banner_normalized(left) == banner_normalized(right),
        "whole G-code differs; {out_dir:?}"
    );
}

fn assert_identities(gcode: &[u8], (machine, process, filament): (&str, &str, &str)) {
    assert_eq!(config(gcode, "printer_settings_id"), machine);
    assert_eq!(config(gcode, "print_settings_id"), process);
    assert_eq!(
        decoded_strings(gcode, "filament_settings_id"),
        vec![filament]
    );
}

fn assert_inherits(gcode: &[u8]) {
    assert_eq!(
        decoded_strings(gcode, "inherits_group"),
        vec![PROCESS, FILAMENT, MACHINE]
    );
}

fn assert_values(gcode: &[u8], expected: [&str; 3]) {
    for (key, value) in KEYS.into_iter().zip(expected) {
        assert_eq!(config(gcode, key), value, "mismatch on {key}");
    }
}

fn decoded_strings(gcode: &[u8], key: &str) -> Vec<String> {
    serde_json::from_str(&decode_config_value_by_key(key, config(gcode, key)).unwrap()).unwrap()
}

fn config<'a>(gcode: &'a [u8], key: &str) -> &'a str {
    let prefix = format!("; {key} = ");
    std::str::from_utf8(gcode)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
}
