//! `CoolingBuffer.cpp:890-898,1000-1002`: restore retained role fan speed.
use super::{ExportedCase, OrcaRunner, cache_identity};

#[test]
fn tiq2_restores_retained_role_fan_complete_sequence() {
    let input = include_bytes!("../../../../../tests/parity/validation/tiq2-type2-enabled.3mf");
    assert_eq!(
        cache_identity::raw_digest(input),
        "0435346ed81fff32acdae7a3a3f2c9e880f4f1efe37d99d3427833836b901942"
    );
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("TiQ2 fan comparison: no optional OrcaRunner");
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("tiq2-retained-fan-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = crate::print_validation::tiq2_corner::corner_input(
        input,
        crate::print_validation::tiq2_corner::Scenario::RetainedFan,
        &directory,
    );
    let path = directory.join("input.3mf");
    std::fs::write(&path, &input).unwrap();
    let output_dir = directory.join("native");
    std::fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: "TiQ2/retained-fan/whole-file".to_owned(),
            project: input.to_vec(),
            project_path: path,
            output_dir,
        })
        .unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            &input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(directory.join("ares.gcode"), &actual).unwrap();
    std::fs::write(directory.join("orca.gcode"), &case.reference).unwrap();
    let reference = fan_records(&case.reference);
    let actual = fan_records(&actual);
    let evidence = |records: &[FanRecord]| {
        records
            .iter()
            .map(|record| {
                (
                    record.line,
                    record.layer,
                    record.command.clone(),
                    record.previous_type.clone(),
                    record.next_type.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    std::fs::write(
        directory.join("fan-context.json"),
        serde_json::to_vec_pretty(&(evidence(&reference), evidence(&actual))).unwrap(),
    )
    .unwrap();
    // Absolute byte/line offsets after unrelated travel differences are evidence,
    // not fan semantics; retain them above and compare every command and context.
    let semantics = |records: &[FanRecord]| {
        records
            .iter()
            .map(|record| {
                (
                    record.command.clone(),
                    record.layer,
                    record.previous_type.clone(),
                    record.next_type.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(semantics(&actual), semantics(&reference), "{directory:?}");
}

struct FanRecord {
    line: usize,
    command: String,
    layer: usize,
    previous_type: String,
    next_type: String,
}

fn fan_records(bytes: &[u8]) -> Vec<FanRecord> {
    let text = std::str::from_utf8(bytes).unwrap();
    let lines = text.lines().collect::<Vec<_>>();
    let mut layer = 0;
    let mut previous_type = "";
    let mut records = Vec::new();
    for (index, line) in lines.iter().copied().enumerate() {
        if line == ";LAYER_CHANGE" {
            layer += 1;
        }
        if line.starts_with(";TYPE:") {
            previous_type = line;
        }
        if matches!(line.split_whitespace().next(), Some("M106" | "M107")) {
            records.push(FanRecord {
                line: index + 1,
                command: line.to_owned(),
                layer,
                previous_type: previous_type.to_owned(),
                next_type: lines[index + 1..]
                    .iter()
                    .find(|line| line.starts_with(";TYPE:"))
                    .copied()
                    .unwrap_or("")
                    .to_owned(),
            });
        }
    }
    records
}
