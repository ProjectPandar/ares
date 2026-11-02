//! PresetBundle.cpp::construct_full_config / CLI::run loaded-system group metadata.
//! Failures: source drift, admission, incomplete/unstable Native, non-neutral
//! control, API/CLI disagreement, effective-config or complete-stream divergence.
use super::{OrcaRunner, cache_identity, profile_entry, stages};
use ares_core::{ConfigOverrides, LoadSettingsFile, LoadSettingsSliceRequest};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

struct Recorded {
    root: PathBuf,
    native_sha: String,
    api_sha: String,
    difference: Option<String>,
}

#[test]
fn loaded_system_inherits_group_preserves_cli_source_status() {
    if !super::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let positive = check("positive", 1);
    let repeat = check("positive", 2);
    assert_eq!(
        positive.native_sha, repeat.native_sha,
        "fresh Native repeat"
    );
    assert_eq!(positive.api_sha, repeat.api_sha, "public API raw repeat");
    assert_eq!(
        positive.difference, repeat.difference,
        "full boundary repeat"
    );
    let control = check("control", 1);
    profile_entry::write_json(
        &control.root.join("loaded-system-summary.json"),
        &json!({"positive":positive.root,"repeat":repeat.root,"control":control.root,
            "comparisons":3,"native_repeat":"PASS","api_repeat":"PASS",
            "native_control_neutral":positive.native_sha==control.native_sha}),
    );
    assert_eq!(
        positive.native_sha, control.native_sha,
        "Native system-parent neutrality"
    );
    assert!(
        control.difference.is_none(),
        "control: {:?}; {:?}",
        control.difference,
        control.root
    );
    assert!(
        positive.difference.is_none(),
        "positive: {:?}; {:?}",
        positive.difference,
        positive.root
    );
    assert!(
        repeat.difference.is_none(),
        "repeat: {:?}; {:?}",
        repeat.difference,
        repeat.root
    );
}

fn check(case: &str, repeat: u8) -> Recorded {
    let mut runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native loaded-settings oracle");
    runner.cache = tempfile::Builder::new()
        .prefix("loaded-system-fresh-native-cache-")
        .tempdir_in(&runner.work)
        .unwrap()
        .keep();
    let fixtures = super::repo_root().join("tests/parity/loaded-system-inherits-group");
    let manifest_bytes = fs::read(fixtures.join("manifest.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let input = fs::read(fixtures.join("single.3mf")).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&input),
        manifest["sha256"]["single.3mf"]
    );
    fs::write(runner.work.join("single.3mf"), &input).unwrap();
    let texts: [String; 3] = ["machine", "process", "filament"].map(|kind| {
        let relative = format!("{case}/{kind}.json");
        let bytes = fs::read(fixtures.join(&relative)).unwrap();
        assert_eq!(
            cache_identity::raw_digest(&bytes),
            manifest["sha256"][&relative]
        );
        fs::write(runner.work.join(format!("{kind}.json")), &bytes).unwrap();
        let fields: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(fields["from"], "system");
        String::from_utf8(bytes).unwrap()
    });
    fs::write(runner.work.join("manifest.json"), &manifest_bytes).unwrap();
    let paths =
        ["machine.json", "process.json", "filament.json"].map(|name| runner.work.join(name));
    let native = runner
        .slice_stl_with_settings(
            [&paths[0], &paths[1], &paths[2]],
            &runner.work.join("single.3mf"),
        )
        .unwrap();
    fs::write(runner.work.join("native.gcode"), &native).unwrap();
    let settings = [
        LoadSettingsFile {
            label: "machine.json",
            json: &texts[0],
        },
        LoadSettingsFile {
            label: "process.json",
            json: &texts[1],
        },
    ];
    let filaments = [LoadSettingsFile {
        label: "filament.json",
        json: &texts[2],
    }];
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_load_settings(
            &input,
            LoadSettingsSliceRequest {
                filename: "single.3mf",
                settings: &settings,
                filaments: &filaments,
                plate: None,
                overrides: &ConfigOverrides::default(),
                placement: ares_core::Placement::None,
                sliced_plate: &mut ares_core::SlicedPlate::default(),
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ));
    if let Err(error) = &api {
        profile_entry::write_json(
            &runner.work.join("api-error.json"),
            &json!({"error":format!("{error:?}")}),
        );
    }
    let api = api.expect("public load-settings API admission");
    fs::write(runner.work.join("ares.gcode"), &api).unwrap();
    stages::run(
        std::path::Path::new(env!("CARGO_BIN_EXE_ares")),
        [
            "slice".to_owned(),
            "--load-settings".to_owned(),
            format!("{};{}", paths[0].display(), paths[1].display()),
            "--load-filaments".to_owned(),
            paths[2].to_str().unwrap().to_owned(),
            "-o".to_owned(),
            runner.work.join("cli.gcode").to_str().unwrap().to_owned(),
            runner.work.join("single.3mf").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Application,
        &runner.work,
    )
    .unwrap();
    let cli = fs::read(runner.work.join("cli.gcode")).unwrap();
    let native_config = config(&native);
    let api_config = config(&api);
    let cli_config = config(&cli);
    let difference = crate::golden::compare_ordered_bytes_generator_only(&native, &api).err();
    let cli_difference = crate::golden::compare_ordered_bytes_generator_only(&native, &cli).err();
    let normalized =
        crate::golden::normalize_one_generator_line(&native, crate::golden::GeneratorKind::Orca)
            .unwrap();
    let native_sha = cache_identity::raw_digest(&normalized);
    let api_sha = cache_identity::raw_digest(&api);
    profile_entry::write_json(
        &runner.work.join("loaded-system-result.json"),
        &json!({"case":case,"repeat":repeat,"approved_spec":manifest["approved_spec"],
            "input_sha256":cache_identity::raw_digest(&input),"input_sources":manifest["sha256"],
            "manifest_sha256":cache_identity::raw_digest(&manifest_bytes),
            "native_sha256":cache_identity::raw_digest(&native),"native_normalized_sha256":native_sha,
            "ares_sha256":api_sha,"cli_sha256":cache_identity::raw_digest(&cli),
            "api_cli_raw_equal":api==cli,"detail":difference,"cli_detail":cli_difference,
            "status":if difference.is_none(){"PASS"}else{"DIVERGENT"},
            "native_config":native_config,"api_config":api_config,"cli_config":cli_config,
            "fresh_native_cache":runner.cache,"rust_entry":"ares_core::slice_with_load_settings",
            "comparison":"complete ordered bytes; generator identity only; motion/arcs/M73/stats/footer retained",
            "evidence":crate::artifacts::EVIDENCE}),
    );
    assert_eq!(api, cli, "API/CLI raw equality; {:?}", runner.work);
    assert_eq!(difference, cli_difference, "public entry boundary equality");
    assert_eq!(
        native_config["inherits_group"], ";;",
        "system-source Native metadata"
    );
    Recorded {
        root: runner.work,
        native_sha,
        api_sha,
        difference,
    }
}

fn config(bytes: &[u8]) -> BTreeMap<String, String> {
    let text = std::str::from_utf8(bytes).unwrap();
    assert!(
        text.lines().any(|line| line == "; CONFIG_BLOCK_END"),
        "complete footer"
    );
    text.lines()
        .skip_while(|line| *line != "; CONFIG_BLOCK_START")
        .skip(1)
        .take_while(|line| *line != "; CONFIG_BLOCK_END")
        .map(|line| line.strip_prefix("; ").unwrap().split_once(" = ").unwrap())
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}
