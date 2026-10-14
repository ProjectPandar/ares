//! M853 E2E: `GCodeProcessorResult::moves` parity against the upstream
//! `GCodeProcessor` oracle (`tools/chain-probe/procgcode.nix`).
//!
//! The oracle re-parses the Ares G-code with `process_file` (no post-process),
//! so its `gcode_id` is the 1-based line id of the final file — the same id
//! upstream's `ExportLines::synchronize_moves` (`GCodeProcessor.cpp:951`)
//! assigns during a real slice. `ARES_PROCGCODE_BIN` points at the oracle's
//! `bin/orca-slicer`; the test skips when it is unset, like `ARES_ORCA_BIN`.
//!
//! Artifacts land in `target/gcode-moves-oracle/<fixture>/`: both CSVs, the
//! oracle input G-code, and `diff.csv` with every out-of-tolerance field.

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const HEADER: &str = "gcode_id,type,role,extruder_id,cp_color_id,x,y,z,delta_extruder,feedrate,\
width,height,mm3_per_mm,travel_dist,fan_speed,temperature,time0,layer_id,internal_only";

/// Per-field tolerance. Fixed by the M853 spec; never loosen — an upstream
/// mismatch beyond these bounds is escalated, not absorbed.
#[derive(Clone, Copy)]
enum Rule {
    /// Integer fields and values upstream copies verbatim from a G-code word
    /// or a single f32 product (`m_feedrate`, `m_fan_speed`,
    /// `m_extruder_temps`): bit-identical.
    Integer,
    /// Verbatim f32 values must have identical bits.
    Exact,
    /// One f32 rounding step: values upstream derives from f64 state narrowed
    /// to f32 (`Vec3f` position, `delta_extruder`, `m_travel_dist`) or from a
    /// short f32 formula (`width`/`height`/`mm3_per_mm`, `:3880-3945`).
    Ulp1,
    /// `time[Normal]` is `block.time()` (`:463`), an f32 trapezoid evaluation
    /// of planner state.
    Ulp2,
}

const RULES: [(&str, Rule); 19] = [
    ("gcode_id", Rule::Integer),
    ("type", Rule::Integer),
    ("role", Rule::Integer),
    ("extruder_id", Rule::Integer),
    ("cp_color_id", Rule::Integer),
    ("x", Rule::Ulp1),
    ("y", Rule::Ulp1),
    ("z", Rule::Ulp1),
    ("delta_extruder", Rule::Ulp1),
    ("feedrate", Rule::Exact),
    ("width", Rule::Ulp1),
    ("height", Rule::Ulp1),
    ("mm3_per_mm", Rule::Ulp1),
    ("travel_dist", Rule::Ulp1),
    ("fan_speed", Rule::Exact),
    ("temperature", Rule::Exact),
    ("time0", Rule::Ulp2),
    ("layer_id", Rule::Integer),
    ("internal_only", Rule::Integer),
];

fn workspace_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn oracle_bin() -> Option<PathBuf> {
    native_environment::binary("ARES_PROCGCODE_BIN")
}

#[test]
fn ender3_moves_match_gcode_processor_oracle() {
    if let Some(oracle) = oracle_bin() {
        check_fixture(
            &oracle,
            "ender3",
            &workspace_path("tests/parity/orca_cli_ender3.3mf"),
            Some(1),
        );
    }
}

#[test]
fn ksr_fdmtest_v4_moves_match_gcode_processor_oracle() {
    if let Some(oracle) = oracle_bin() {
        check_fixture(
            &oracle,
            "ksr_fdmtest_v4",
            &workspace_path("tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf"),
            None,
        );
    }
}

/// `OptionsZCorrector::update` (`GCodeProcessor.hpp:693-705`) erases and
/// relocates the vertex at its stale index after actual-speed insertions
/// (`GCodeProcessor.cpp:5951-6009`); see
/// `docs/architecture/gcode-processor-options-z-moves.md`.
fn check_emission_fixture(name: &str) {
    if let Some(oracle) = oracle_bin() {
        check_fixture(
            &oracle,
            name,
            &workspace_path(&format!(
                "tests/parity/plate-custom-gcode-emission/{name}.3mf"
            )),
            None,
        );
    }
}

#[test]
fn plate_custom_gcode_template_moves_match_gcode_processor_oracle() {
    check_emission_fixture("template");
}

#[test]
fn plate_custom_gcode_custom_moves_match_gcode_processor_oracle() {
    check_emission_fixture("custom");
}

#[test]
fn plate_custom_gcode_pause_moves_match_gcode_processor_oracle() {
    check_emission_fixture("pause");
}

#[test]
fn plate_custom_gcode_mode_mismatch_moves_match_gcode_processor_oracle() {
    check_emission_fixture("mode-mismatch");
}

#[test]
fn plate_custom_gcode_same_layer_moves_match_gcode_processor_oracle() {
    check_emission_fixture("same-layer");
}

fn check_fixture(oracle: &Path, name: &str, project: &Path, plate: Option<u32>) {
    let artifacts = workspace_path("target/gcode-moves-oracle").join(name);
    fs::create_dir_all(&artifacts).unwrap();
    let gcode = artifacts.join("ares.gcode");
    let ares_csv = artifacts.join("ares.csv");
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("ares");
    command.args(["slice", "--dump-moves"]).arg(&ares_csv);
    if let Some(plate) = plate {
        command.args(["--plate", &plate.to_string()]);
    }
    command
        .arg("-o")
        .arg(&gcode)
        .arg(project)
        .assert()
        .success();

    // `GCodeProcessor::detect_producer` (`:1691`) only loads the embedded
    // config — and parses HEIGHT/WIDTH tags — for known producers. Renaming
    // the producer keeps the line count, so every `gcode_id` is unchanged.
    let text = fs::read_to_string(&gcode).unwrap();
    let renamed = text.replacen(
        "\n; generated by Ares 2.4.2\n",
        "\n; generated by OrcaSlicer 2.4.2\n",
        1,
    );
    assert_ne!(renamed, text, "Ares producer line not found");
    let oracle_input = artifacts.join("oracle-input.gcode");
    fs::write(&oracle_input, renamed).unwrap();
    let oracle_csv = artifacts.join("oracle.csv");
    let status = Command::new(oracle)
        .arg("--process-gcode")
        .arg(&oracle_input)
        .arg("--dump-moves")
        .arg(&oracle_csv)
        .status()
        .unwrap();
    assert!(status.success(), "oracle failed: {status}");

    let expected = read_moves(&oracle_csv);
    let actual = read_moves(&ares_csv);
    let report = compare(&expected, &actual);
    fs::write(artifacts.join("diff.csv"), &report.diff).unwrap();
    assert_eq!(
        report.mismatches,
        0,
        "{name}: {} out-of-tolerance fields over {} oracle / {} Ares moves; see {}",
        report.mismatches,
        expected.len(),
        actual.len(),
        artifacts.join("diff.csv").display(),
    );
}

fn read_moves(path: &Path) -> Vec<Vec<String>> {
    let text = fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some(HEADER), "{}", path.display());
    lines
        .map(|line| line.split(',').map(str::to_owned).collect::<Vec<_>>())
        .inspect(|row| assert_eq!(row.len(), 19, "{}", path.display()))
        .collect()
}

struct Report {
    mismatches: usize,
    diff: String,
}

fn check(rule: &Rule, oracle: &str, ares: &str) -> (bool, String) {
    let bound = match rule {
        Rule::Integer => {
            return (
                oracle.parse::<u32>().unwrap() == ares.parse::<u32>().unwrap(),
                String::new(),
            );
        }
        Rule::Exact => return (oracle == ares || f32_eq(oracle, ares), String::new()),
        Rule::Ulp1 => 1,
        Rule::Ulp2 => 2,
    };
    let ulp = ulp_distance(oracle, ares);
    (ulp <= bound, ulp.to_string())
}

fn compare(expected: &[Vec<String>], actual: &[Vec<String>]) -> Report {
    let mut diff = String::from("move,field,oracle,ares,ulp\n");
    let mut mismatches = 0;
    for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
        for (field, (name, rule)) in RULES.iter().enumerate() {
            let (oracle, ares) = (&expected[field], &actual[field]);
            let (within, ulp) = check(rule, oracle, ares);
            if !within {
                mismatches += 1;
                writeln!(diff, "{index},{name},{oracle},{ares},{ulp}").unwrap();
            }
        }
    }
    if expected.len() != actual.len() {
        mismatches += 1;
        writeln!(diff, "-,move_count,{},{},", expected.len(), actual.len()).unwrap();
    }
    Report { mismatches, diff }
}

#[test]
fn exact_move_ids_do_not_round_through_f32() {
    let mut expected = vec![vec!["0".to_owned(); RULES.len()]];
    expected[0][0] = "16777216".into();
    let mut actual = expected.clone();
    actual[0][0] = "16777217".into();
    assert_eq!(compare(&expected, &actual).mismatches, 1);
}

fn parse(value: &str) -> f32 {
    value
        .parse()
        .unwrap_or_else(|_| panic!("not a float: {value}"))
}

fn f32_eq(left: &str, right: &str) -> bool {
    parse(left).to_bits() == parse(right).to_bits()
}

/// Distance in representable f32 values (`-0.0` and `+0.0` share ordinal 0).
fn ulp_distance(left: &str, right: &str) -> u64 {
    let ordered = |value: f32| {
        let bits = value.to_bits() as i32;
        i64::from(if bits < 0 { i32::MIN - bits } else { bits })
    };
    ordered(parse(left)).abs_diff(ordered(parse(right)))
}
