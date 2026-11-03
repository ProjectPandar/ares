use super::super::{native_api_cli::sha, native_stream_case, repo_root};
use crate::golden;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::{collections::BTreeSet, fs};

#[test]
fn native_full_config_block_matches_complete_public_output() {
    let project = fs::read(repo_root().join("tests/parity/orca_cli_ender3.3mf")).unwrap();
    let (reference, replay) = native_stream_case::qualify(
        (
            "config-block-keyset",
            &[
                "GCode.cpp:8244-8354 append_full_config (345efec64c7b)",
                "PrintConfig.cpp:10002-10035 obsolete input keys",
            ],
        ),
        "historical-ender3",
        &project,
        "7fb06c23baed9c8e06757cc0f7128c827e0b232eaa2655af687e4708014037c5",
        false,
    );
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ));
    fs::write(
        reference.root.join("api-admission.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": sha(&project),
            "api_error": result.as_ref().err().map(|error| format!("{error:?}")),
        }))
        .unwrap(),
    )
    .unwrap();
    let api =
        result.unwrap_or_else(|error| panic!("API admission: {error:?}; {:?}", reference.root));
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let native = block(&reference.native);
    let replay = block(&replay.native);
    let api = block(std::str::from_utf8(&api).unwrap());
    assert_eq!(native, replay, "complete Native block replay");
    fs::write(reference.root.join("native-config-block.txt"), native).unwrap();
    fs::write(reference.root.join("ares-config-block.txt"), api).unwrap();
    let native_keys = keys(native);
    let api_keys = keys(api);
    let difference = golden::first_difference(native.as_bytes(), api.as_bytes())
        .map(|difference| difference.to_string());
    fs::write(
        reference.root.join("config-block-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "scope": "entire CONFIG_BLOCK_START through CONFIG_BLOCK_END newline, unmodified bytes",
            "native_bytes": native.len(), "api_bytes": api.len(),
            "native_sha256": sha(native.as_bytes()), "api_sha256": sha(api.as_bytes()),
            "native_records": native.lines().count() - 2,
            "api_records": api.lines().count() - 2,
            "native_unique_keys": native_keys.len(), "api_unique_keys": api_keys.len(),
            "missing_keys": native_keys.difference(&api_keys).collect::<Vec<_>>(),
            "extra_keys": api_keys.difference(&native_keys).collect::<Vec<_>>(),
            "first_difference": difference,
            "diagnostics_do_not_filter_acceptance": true,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        difference.is_none(),
        "complete CONFIG_BLOCK: {}; {:?}",
        difference.unwrap_or_default(),
        reference.root
    );
}

fn block(text: &str) -> &str {
    const START: &str = "; CONFIG_BLOCK_START\n";
    const END: &str = "; CONFIG_BLOCK_END\n";
    assert_eq!(text.matches(START).count(), 1);
    assert_eq!(text.matches(END).count(), 1);
    let start = text.find(START).unwrap();
    let end = text[start..].find(END).unwrap() + start + END.len();
    &text[start..end]
}

fn keys(block: &str) -> BTreeSet<&str> {
    block
        .lines()
        .filter_map(|line| {
            line.strip_prefix("; ")?
                .split_once(" = ")
                .map(|(key, _)| key)
        })
        .collect()
}
