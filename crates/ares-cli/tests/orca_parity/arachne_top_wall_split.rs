use crate::{
    golden,
    runner::{OrcaRunner, application, stages},
};
use std::{collections::BTreeMap, fs, path::Path};

const GUIDER_STOCK: &[u8] =
    include_bytes!("../../../../tests/parity/arachne-top-wall-split/guider-stock.3mf");
const GUIDER_ALL_WALLS: &[u8] =
    include_bytes!("../../../../tests/parity/arachne-top-wall-split/guider-all-walls.3mf");
const MK4_STOCK: &[u8] =
    include_bytes!("../../../../tests/parity/arachne-top-wall-split/mk4-stock.3mf");
const MK4_SINGLE_TOP_WALL: &[u8] =
    include_bytes!("../../../../tests/parity/arachne-top-wall-split/mk4-single-top-wall.3mf");

#[test]
fn guider_arachne_keeps_walls_outside_single_wall_top_surfaces() {
    compare("guider", "3.47", [GUIDER_ALL_WALLS, GUIDER_STOCK]);
}

#[test]
fn mk4_arachne_keeps_walls_outside_single_wall_top_surfaces() {
    compare("mk4", "3.4", [MK4_STOCK, MK4_SINGLE_TOP_WALL]);
}

fn compare(machine: &str, height: &str, fixtures: [&[u8]; 2]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("single top wall requires Native OrcaSlicer");
        return;
    };
    let catalog = std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG")
        .expect("single top wall projects require the project preset catalog");
    let catalog = Path::new(&catalog);
    let root = tempfile::Builder::new()
        .prefix(&format!("arachne-single-top-wall-{machine}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let references = [("0", "all-walls"), ("1", "single-top-wall")]
        .into_iter()
        .zip(fixtures)
        .map(|((enabled, label), input)| {
            let dir = root.join(label);
            fs::create_dir(&dir).unwrap();
            let native = native(&runner, &dir, catalog, input);
            let text = std::str::from_utf8(&native).unwrap();
            for line in [
                "; wall_generator = arachne",
                &format!("; only_one_wall_top = {enabled}"),
            ] {
                assert!(text.lines().any(|s| s == line), "{line}: {dir:?}");
            }
            let walls = inner_wall_segments(text, height);
            assert!(
                !walls.is_empty(),
                "no Native inner walls at {height}: {dir:?}"
            );
            (dir, native, walls)
        })
        .collect::<Vec<_>>();
    assert_ne!(
        references[0].2, references[1].2,
        "Native top split must change inner wall geometry at {height}: {root:?}"
    );
    let mut failures = Vec::new();
    for ((dir, reference, _), input) in references.iter().zip(fixtures) {
        failures.extend(compare_public(dir, catalog, input, reference));
    }
    assert!(failures.is_empty(), "{failures:?}; artifacts: {root:?}");
}

fn native(runner: &OrcaRunner, dir: &Path, catalog: &Path, input: &[u8]) -> Vec<u8> {
    let datadir = catalog.parent().unwrap().join("native");
    let out = dir.join("native");
    let project = dir.join("input.3mf");
    fs::create_dir(&out).unwrap();
    fs::write(&project, input).unwrap();
    stages::run(
        runner.bin(),
        [
            "--datadir",
            datadir.to_str().unwrap(),
            "--slice",
            "1",
            "--outputdir",
            out.to_str().unwrap(),
            project.to_str().unwrap(),
        ]
        .map(str::to_owned),
        stages::Stage::Slice,
        dir,
    )
    .unwrap();
    fs::read(out.join("plate_1.gcode")).unwrap()
}

fn compare_public(dir: &Path, catalog: &Path, input: &[u8], reference: &[u8]) -> Vec<String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let api = runtime
        .block_on(application::slice_project_plate_validated(
            input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .map_err(|e| format!("{e:?}"));
    let output = dir.join("ares.cli.gcode");
    let cli = application::CliCommand::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--plate", "1", "-o"])
        .arg(&output)
        .arg(dir.join("input.3mf"))
        .arg("--project-preset-catalog")
        .arg(catalog)
        .output()
        .unwrap();
    fs::write(dir.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let emitted = if cli.status.success() {
        fs::read(output).map_err(|e| e.to_string())
    } else {
        Err(format!("CLI returned {}", cli.status))
    };
    if let Ok(bytes) = &api {
        fs::write(dir.join("ares.api.gcode"), bytes).unwrap();
    }
    [("API", api), ("CLI", emitted)]
        .into_iter()
        .filter_map(|(seam, result)| {
            result
                .and_then(|bytes| golden::compare_ordered_bytes_generator_only(reference, &bytes))
                .err()
                .map(|error| format!("{} {seam}: {error}", dir.display()))
        })
        .collect()
}

fn inner_wall_segments(text: &str, height: &str) -> BTreeMap<[[i64; 2]; 2], usize> {
    let mut result = BTreeMap::new();
    let mut active_height = false;
    let mut inner_wall = false;
    let mut pos = [0; 2];
    let mut e = 0.0;
    let mut relative_e = false;
    for line in text.lines() {
        if let Some(z) = line
            .strip_prefix("; Z_HEIGHT: ")
            .or_else(|| line.strip_prefix(";Z:"))
        {
            active_height = z == height;
        }
        if let Some(role) = line
            .strip_prefix("; FEATURE: ")
            .or_else(|| line.strip_prefix(";TYPE:"))
        {
            inner_wall = role == "Inner wall";
        }
        let mut words = line.split(';').next().unwrap().split_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        match command {
            "M82" | "M83" => {
                relative_e = command == "M83";
                continue;
            }
            "G0" | "G1" | "G92" => {}
            _ => continue,
        }
        let start = pos;
        let mut e_value = None;
        for word in words {
            let (axis, value) = word.split_at(1);
            let Ok(value) = value.parse::<f64>() else {
                continue;
            };
            match axis {
                "X" if command != "G92" => pos[0] = (1000.0 * value).round() as i64,
                "Y" if command != "G92" => pos[1] = (1000.0 * value).round() as i64,
                "E" => e_value = Some(value),
                _ => {}
            }
        }
        let Some(value) = e_value else { continue };
        let delta = if relative_e { value } else { value - e };
        if !relative_e || command == "G92" {
            e = value;
        }
        if command == "G1" && active_height && inner_wall && delta > 0.0 && start != pos {
            let mut segment = [start, pos];
            segment.sort_unstable();
            *result.entry(segment).or_default() += 1;
        }
    }
    result
}
