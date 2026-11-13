//! Final2.5 a2acec57: FilamentGroup.cpp:245-270,472-495,571-623.
//! Keep the historical test ID; both flush-entry endpoints now select the primary.
use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use std::{fs, io::Cursor};

const CASES: [(&str, &str, [i32; 3]); 2] = [
    (
        "x2d_pla_three_filaments_remembered_small_gap",
        "80e2db05444d4b78ba663ea374141537fa241027c67c4dd6a6ea6ef6d201f19a",
        [1, 2, 1],
    ),
    (
        "x2d_pla_three_filaments_outside_absolute_gap",
        "b02152095c3302ed9447fea555581e2f6dc1c9e8e052536640181a1ebbae9fb2",
        [1, 2, 1],
    ),
];

struct Observed {
    native: [i32; 3],
    api: [i32; 3],
    cli: [i32; 3],
    diagnostics: String,
}

fn map(text: &str) -> [i32; 3] {
    text.split_once("; CONFIG_BLOCK_START\n")
        .unwrap()
        .1
        .split_once("; CONFIG_BLOCK_END")
        .unwrap()
        .0
        .lines()
        .find_map(|line| line.strip_prefix("; filament_map = "))
        .unwrap()
        .split(',')
        .map(|value| value.parse::<i32>().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn sample((name, digest, expected): (&str, &str, [i32; 3]), input: &[u8]) -> Observed {
    let (reference, _) = native_stream_case::qualify(
        (
            "filament-group-flush-remembered-pair",
            &[
                "FilamentGroup.cpp:472-495 empty-inventory first-candidate selection",
                "FilamentGroup.cpp:245-270,571-623 palette-relative retained budget",
                "FilamentGroup.cpp:1313-1329 primary and remembered candidate composition",
            ],
        ),
        name,
        input,
        digest,
        false,
    );
    let native = map(&reference.native);
    assert_eq!(
        native, expected,
        "Native endpoint map; {:?}",
        reference.root
    );
    let usage = reference
        .native
        .lines()
        .find_map(|line| line.strip_prefix("; filament used [mm] = "))
        .unwrap()
        .split(", ")
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(usage.len(), 3);
    assert!(usage.iter().all(|value| *value > 0.0));
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            input,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = reference.root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(
        cli.status.success(),
        "CLI at {:?}: {:?}; stderr={}; stdout={}",
        reference.root,
        cli.status,
        String::from_utf8_lossy(&cli.stderr),
        String::from_utf8_lossy(&cli.stdout)
    );
    let cli = fs::read(cli_path).unwrap();
    assert_eq!(api, cli, "complete API/CLI bytes; {:?}", reference.root);
    let strict_api =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &api);
    let strict_cli =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &cli);
    fs::write(
        reference.root.join("remembered-map-comparison.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "input_sha256": digest, "native": native,
            "api": map(std::str::from_utf8(&api).unwrap()),
            "cli": map(std::str::from_utf8(&cli).unwrap()),
            "api_sha256": sha(&api), "cli_sha256": sha(&cli),
            "strict_whole_api": strict_api, "strict_whole_cli": strict_cli,
            "whole_output_parity_claimed": false
        }))
        .unwrap(),
    )
    .unwrap();
    Observed {
        native,
        api: map(std::str::from_utf8(&api).unwrap()),
        cli: map(std::str::from_utf8(&cli).unwrap()),
        diagnostics: format!(
            "input={digest}; {:?}; api_sha256={}; cli_sha256={}; strict_whole_api={strict_api:?}; strict_whole_cli={strict_cli:?}",
            reference.root,
            sha(&api),
            sha(&cli)
        ),
    }
}

#[test]
fn x2d_pla_remembered_gap_changes_empty_inventory_assignment() {
    let root = repo_root().join("tests/parity/filament-group-flush-mechanisms");
    let inputs = CASES.map(|(name, digest, _)| {
        let bytes = fs::read(root.join(format!("{name}.3mf"))).unwrap();
        assert_eq!(sha(&bytes), digest, "fixture input");
        bytes
    });
    let mut small = zip::ZipArchive::new(Cursor::new(&inputs[0])).unwrap();
    let mut outside = zip::ZipArchive::new(Cursor::new(&inputs[1])).unwrap();
    assert_eq!(small.len(), outside.len());
    for index in 0..small.len() {
        let mut first = small.by_index(index).unwrap();
        let mut second = outside.by_index(index).unwrap();
        assert_eq!(first.name(), second.name());
        let mut first_bytes = Vec::new();
        let mut second_bytes = Vec::new();
        std::io::Read::read_to_end(&mut first, &mut first_bytes).unwrap();
        std::io::Read::read_to_end(&mut second, &mut second_bytes).unwrap();
        if first.name() == "Metadata/project_settings.config" {
            let mut a: serde_json::Value = serde_json::from_slice(&first_bytes).unwrap();
            let mut b: serde_json::Value = serde_json::from_slice(&second_bytes).unwrap();
            let matrix_a = a
                .as_object_mut()
                .unwrap()
                .remove("flush_volumes_matrix")
                .unwrap();
            let matrix_b = b
                .as_object_mut()
                .unwrap()
                .remove("flush_volumes_matrix")
                .unwrap();
            assert_ne!(matrix_a, matrix_b, "flush entries must change");
            assert_eq!(a, b, "all other settings unchanged");
        } else {
            assert_eq!(first_bytes, second_bytes, "same model/plate and presets");
        }
    }
    let endpoints = [sample(CASES[0], &inputs[0]), sample(CASES[1], &inputs[1])];
    let native = endpoints.each_ref().map(|endpoint| endpoint.native);
    let api = endpoints.each_ref().map(|endpoint| endpoint.api);
    let cli = endpoints.each_ref().map(|endpoint| endpoint.cli);
    let delta = |maps: [[i32; 3]; 2]| std::array::from_fn::<_, 3, _>(|i| maps[1][i] - maps[0][i]);
    let native_delta = delta(native);
    let api_delta = delta(api);
    let cli_delta = delta(cli);
    assert_eq!(
        native_delta,
        [0, 0, 0],
        "final2.5 empty inventory keeps both primaries"
    );
    assert_eq!(
        api, native,
        "API endpoint maps; native_delta={native_delta:?}; api_delta={api_delta:?}; cli_delta={cli_delta:?}; small_gap={}; outside_gap={}",
        endpoints[0].diagnostics, endpoints[1].diagnostics
    );
    assert_eq!(cli, native, "CLI endpoint maps");
    assert_eq!(api_delta, native_delta, "API remembered-gap direction");
    assert_eq!(cli_delta, native_delta, "CLI remembered-gap direction");
}
