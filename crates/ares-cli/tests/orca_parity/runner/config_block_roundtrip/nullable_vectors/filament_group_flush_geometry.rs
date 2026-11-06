//! PrintObject.cpp height, printable-area and exclusion-prism restrictions.
use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use std::{collections::BTreeMap, fs, io::Cursor};

const KEYS: [&str; 14] = [
    "filament_map",
    "filament_volume_map",
    "filament_nozzle_map",
    "filament_self_index",
    "filament_retraction_length",
    "filament_printable",
    "extruder_ams_count",
    "group_algo_with_time",
    "flush_multiplier",
    "flush_volumes_matrix",
    "extruder_printable_height",
    "extruder_printable_area",
    "bed_exclude_volume_mode",
    "extruder_bed_exclude_volumes",
];

fn fields(text: &str) -> BTreeMap<&str, &str> {
    let block = text.split_once("; CONFIG_BLOCK_START\n").unwrap().1;
    let block = block.split_once("; CONFIG_BLOCK_END").unwrap().0;
    let values = block
        .lines()
        .filter_map(|line| line.strip_prefix("; ")?.split_once(" = "))
        .filter(|(key, _)| KEYS.contains(key))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(values.len(), KEYS.len());
    values
}

fn fixture(name: &str) {
    let fixtures = repo_root().join("tests/parity/filament-group-flush-geometry");
    let provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(fixtures.join("provenance.json")).unwrap()).unwrap();
    let case = provenance["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == name)
        .unwrap();
    let input = fs::read(fixtures.join(case["input"].as_str().unwrap())).unwrap();
    let input_sha = case["sha256"].as_str().unwrap();
    assert_eq!(sha(&input), input_sha, "frozen geometry input");
    let control =
        fs::read(fixtures.join(provenance["control"]["input"].as_str().unwrap())).unwrap();
    assert_eq!(
        sha(&control),
        provenance["control"]["sha256"].as_str().unwrap()
    );
    let mut actual_zip = zip::ZipArchive::new(Cursor::new(&input)).unwrap();
    let mut control_zip = zip::ZipArchive::new(Cursor::new(&control)).unwrap();
    assert_eq!(actual_zip.len(), control_zip.len());
    for index in 0..actual_zip.len() {
        let mut actual = actual_zip.by_index(index).unwrap();
        let mut control = control_zip.by_index(index).unwrap();
        assert_eq!(actual.name(), control.name());
        if actual.name() != "Metadata/project_settings.config" {
            let mut actual_bytes = Vec::new();
            let mut control_bytes = Vec::new();
            std::io::Read::read_to_end(&mut actual, &mut actual_bytes).unwrap();
            std::io::Read::read_to_end(&mut control, &mut control_bytes).unwrap();
            assert_eq!(actual_bytes, control_bytes, "unchanged model/plate entries");
        }
    }
    let (reference, _) = native_stream_case::qualify(
        (
            "filament-group-flush-geometry",
            &[
                "PrintObject.cpp:349-515 actual geometric restrictions",
                "Print.cpp:3593-3604,4995-5006 union and printable-area difference",
                "ToolOrdering.cpp:1537-1672 grouping restriction context",
            ],
        ),
        name,
        &input,
        input_sha,
        false,
    );
    let native_fields = fields(&reference.native);
    for key in KEYS {
        assert_eq!(
            native_fields[key],
            case["expected_native"][key].as_str().unwrap(),
            "Native {key}; input={input_sha}; {:?}",
            reference.root
        );
    }
    let usage = reference
        .native
        .lines()
        .find_map(|line| line.strip_prefix("; filament used [mm] = "))
        .unwrap();
    let used = usage
        .split(", ")
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(used.len(), 3);
    assert!(used.iter().all(|value| *value > 0.0));
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &input,
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
    assert_eq!(
        api, cli,
        "complete API/CLI byte equality; {:?}",
        reference.root
    );
    let api_fields = fields(std::str::from_utf8(&api).unwrap());
    let cli_fields = fields(std::str::from_utf8(&cli).unwrap());
    let strict_api =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &api);
    let strict_cli =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &cli);
    for key in KEYS {
        assert_eq!(
            api_fields[key], native_fields[key],
            "API {key}; input={input_sha}; {:?}; strict_whole_api={strict_api:?}; strict_whole_cli={strict_cli:?}",
            reference.root
        );
        assert_eq!(
            cli_fields[key], native_fields[key],
            "CLI {key}; input={input_sha}; {:?}",
            reference.root
        );
    }
}

macro_rules! cases {
    ($($name:ident),+ $(,)?) => {$ (
        #[test]
        fn $name() { fixture(stringify!($name)); }
    )+};
}

cases! {
    x2d_pla_auto_map_first_head_height_below_model,
    x2d_pla_auto_map_first_head_area_excludes_left_object,
    x2d_pla_auto_map_first_head_area_excludes_right_object,
    x2d_pla_auto_map_first_head_exclusion_intersects_left_object,
}
