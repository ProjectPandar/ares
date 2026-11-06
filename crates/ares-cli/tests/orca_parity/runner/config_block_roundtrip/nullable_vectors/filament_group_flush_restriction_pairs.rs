//! PrintObject.cpp:349-515 restriction-induced physical-head assignment changes.
use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use std::{fs, io::Cursor};

const CONTROL: &str = "x2d_pla_both_heads_reach_models.3mf";
const CONTROL_SHA: &str = "717250cd867278868a88c646f9754f7761dd73f1f8d461e965b294545cf3db66";

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

fn sample(name: &str, input: &[u8], digest: &str, expected_native: [i32; 3]) -> Observed {
    let (reference, _) = native_stream_case::qualify(
        (
            "filament-group-flush-restriction-pairs",
            &[
                "PrintObject.cpp:349-515 height and printable-area restrictions",
                "Print.cpp:3593-3604 union of actual filament restrictions",
                "ToolOrdering.cpp:1537-1672 restriction context for grouping",
            ],
        ),
        name,
        input,
        digest,
        false,
    );
    let native = map(&reference.native);
    assert_eq!(
        native, expected_native,
        "Native endpoint map; input={digest}; {:?}",
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
    let cli_path = reference.root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
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
    Observed {
        native,
        api: map(std::str::from_utf8(&api).unwrap()),
        cli: map(std::str::from_utf8(&cli).unwrap()),
        diagnostics: format!(
            "input={digest}; {:?}; strict_whole_api={strict_api:?}; strict_whole_cli={strict_cli:?}",
            reference.root
        ),
    }
}

fn paired(name: &str, restricted: &str, restricted_sha: &str, key: &str) {
    let root = repo_root().join("tests/parity/filament-group-flush-restriction-pairs");
    let control = fs::read(root.join(CONTROL)).unwrap();
    let limited = fs::read(root.join(restricted)).unwrap();
    assert_eq!(sha(&control), CONTROL_SHA, "frozen control");
    assert_eq!(sha(&limited), restricted_sha, "frozen restricted input");
    let mut control_zip = zip::ZipArchive::new(Cursor::new(&control)).unwrap();
    let mut limited_zip = zip::ZipArchive::new(Cursor::new(&limited)).unwrap();
    assert_eq!(control_zip.len(), limited_zip.len());
    for index in 0..control_zip.len() {
        let mut original = control_zip.by_index(index).unwrap();
        let mut changed = limited_zip.by_index(index).unwrap();
        assert_eq!(original.name(), changed.name());
        let mut original_bytes = Vec::new();
        let mut changed_bytes = Vec::new();
        std::io::Read::read_to_end(&mut original, &mut original_bytes).unwrap();
        std::io::Read::read_to_end(&mut changed, &mut changed_bytes).unwrap();
        if original.name() == "Metadata/project_settings.config" {
            let mut before: serde_json::Value = serde_json::from_slice(&original_bytes).unwrap();
            let mut after: serde_json::Value = serde_json::from_slice(&changed_bytes).unwrap();
            assert_ne!(before[key], after[key], "restriction must change");
            before.as_object_mut().unwrap().remove(key);
            after.as_object_mut().unwrap().remove(key);
            assert_eq!(before, after, "all other settings unchanged");
        } else {
            assert_eq!(
                original_bytes, changed_bytes,
                "same model/plate and presets"
            );
        }
    }
    let endpoints = [
        sample(
            &format!("{name}-unrestricted"),
            &control,
            CONTROL_SHA,
            [1, 2, 1],
        ),
        sample(
            &format!("{name}-restricted"),
            &limited,
            restricted_sha,
            [1, 1, 1],
        ),
    ];
    let native_maps = endpoints.each_ref().map(|endpoint| endpoint.native);
    let api_maps = endpoints.each_ref().map(|endpoint| endpoint.api);
    let cli_maps = endpoints.each_ref().map(|endpoint| endpoint.cli);
    let delta = |maps: [[i32; 3]; 2]| std::array::from_fn::<_, 3, _>(|i| maps[1][i] - maps[0][i]);
    let native_delta = delta(native_maps);
    let api_delta = delta(api_maps);
    let cli_delta = delta(cli_maps);
    assert_eq!(native_delta, [0, -1, 0], "Native restriction direction");
    assert_eq!(
        api_maps, native_maps,
        "API endpoint maps; native_delta={native_delta:?}; api_delta={api_delta:?}; cli_delta={cli_delta:?}; unrestricted={}; restricted={}",
        endpoints[0].diagnostics, endpoints[1].diagnostics
    );
    assert_eq!(cli_maps, native_maps, "CLI endpoint maps");
    assert_eq!(api_delta, native_delta, "API restriction direction");
    assert_eq!(cli_delta, native_delta, "CLI restriction direction");
}

#[test]
fn x2d_pla_auto_map_second_head_height_changes_assignment() {
    paired(
        "second-head-height",
        "x2d_pla_second_head_below_models.3mf",
        "6d550d5c4a39d4ae89c98881f183d079c5dc007a1f342648f17fdde57c3d4909",
        "extruder_printable_height",
    );
}

#[test]
fn x2d_pla_auto_map_second_head_area_changes_assignment() {
    paired(
        "second-head-area",
        "x2d_pla_second_head_area_outside_models.3mf",
        "2fd4454175ebf8158a258f504d54308cd8cae9bea4de5486870eb64f147a0314",
        "extruder_printable_area",
    );
}
