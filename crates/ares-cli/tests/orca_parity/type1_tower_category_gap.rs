//! Compare the planned first-layer footprint, not complete tower emission.
use crate::{
    OrcaRunner,
    runner::{application, native_api_cli::NativeCase},
};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use std::{
    fs,
    io::{Cursor, Write},
};

const X2D: &[u8] = include_bytes!(
    "../../../../tests/parity/filament-group-flush-restriction-pairs/x2d_pla_both_heads_reach_models.3mf"
);

fn enable_tower(narrower: bool) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(X2D)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let entry = input.by_index(index).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            output
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            let mut config: serde_json::Value = serde_json::from_reader(entry).unwrap();
            assert_eq!(config["enable_prime_tower"], "0");
            config["enable_prime_tower"] = serde_json::json!("1");
            if narrower {
                assert_eq!(config["prime_tower_width"], "35");
                config["prime_tower_width"] = serde_json::json!("33");
            }
            output
                .write_all(&serde_json::to_vec(&config).unwrap())
                .unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    output.finish().unwrap().into_inner()
}

#[derive(Debug, PartialEq, Eq)]
struct Footprint<'a> {
    bed_leveling_rectangle: Vec<&'a str>,
    outer_wall: Vec<Vec<&'a str>>,
}

fn footprint(gcode: &str) -> Footprint<'_> {
    let rectangle = gcode
        .lines()
        .map(str::trim_start)
        .find(|line| line.starts_with("G29 A1 X"))
        .expect("G-code contains the bed-leveling rectangle");
    let before_change = gcode
        .split_once("\n; CP TOOLCHANGE START\n")
        .expect("Type1 tower has a tool change")
        .0;
    let (_, wall) = before_change
        .split_once("\n; WIPE_TOWER_START\n")
        .expect("Type1 first-layer wall precedes the first tool change");
    let (wall, _) = wall
        .split_once("\n; WIPE_TOWER_END\n")
        .expect("Type1 first-layer wall section is complete");
    let outer_wall = wall
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_ascii_whitespace();
            let command = fields.next()?;
            if !matches!(command, "G1" | "G2" | "G3") {
                return None;
            }
            let mut coordinates = vec![command];
            coordinates.extend(fields.filter(|field| field.starts_with(['X', 'Y', 'I', 'J'])));
            (coordinates.len() > 1).then_some(coordinates)
        })
        .collect::<Vec<_>>();
    assert!(
        !outer_wall.is_empty(),
        "first-layer wall contains positioning moves"
    );
    assert!(
        outer_wall.iter().any(|movement| movement[0] != "G1"),
        "rib outer wall contains arcs"
    );
    Footprint {
        bed_leveling_rectangle: rectangle.split_ascii_whitespace().collect(),
        outer_wall,
    }
}

fn check(narrower: bool, prefix: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input = enable_tower(narrower);
    let case = NativeCase::slice(&runner, &input, prefix);
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            &input,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(case.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = case.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(case.root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "CLI footprint at {:?}: {:?}",
        case.root,
        cli.status
    );
    let cli = fs::read_to_string(cli_path).unwrap();
    let expected = footprint(&case.native);
    for (producer, output) in [
        ("API", std::str::from_utf8(&api).unwrap()),
        ("CLI", cli.as_str()),
    ] {
        let observed = footprint(output);
        assert_eq!(
            expected.bed_leveling_rectangle, observed.bed_leveling_rectangle,
            "{producer} Type1 bed-leveling rectangle at {:?}",
            case.root
        );
        assert_eq!(
            expected.outer_wall, observed.outer_wall,
            "{producer} Type1 first-layer outer-wall coordinates at {:?}",
            case.root
        );
    }
}

#[test]
fn type1_planned_footprint_uses_category_nozzle_gap() {
    check(false, "type1-cross-head-category-nozzle-gap-");
}

#[test]
fn type1_planned_footprint_uses_replan_nozzle_opening() {
    check(true, "type1-cross-head-replan-opening-");
}
