use super::{OrcaRunner, project, runner};

fn compare(case: &str, ratio: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = project(&runner, case);
    runner::native_stream_case::check(
        ("top-flow", &["GCode.cpp:9475–9485", "Fill.cpp:330–372"]),
        case,
        &project,
        &runner::native_api_cli::sha(&project),
        (false, |reference, _| {
            let text = &reference.native;
            assert!(
                text.lines()
                    .any(|line| line == format!("; top_solid_infill_flow_ratio = {ratio}")),
                "finalized top-flow vector: {:?}",
                reference.root
            );
            if case != "top_flow_singleton" {
                assert!(text.contains("\n; filament_map = 2\n"));
                assert!(text.contains("\n; filament_map_mode = Manual\n"));
                assert!(text.contains("\n; print_extruder_id = 7,7\n"));
            }
            assert!(
                text.split("; FEATURE: Top surface\n").skip(1).any(|block| {
                    block
                        .lines()
                        .take_while(|line| !line.starts_with("; FEATURE:"))
                        .any(|line| {
                            line.starts_with("G1 X") && line.contains(" E") && !line.contains(" E-")
                        })
                }),
                "positive top extrusion: {:?}",
                reference.root
            );
        }),
    );
}

#[test]
fn unmatched_process_id_uses_top_flow_column() {
    compare("top_flow_unmatched", "0.87,1.13");
}

#[test]
fn equal_top_flow_columns_preserve_output() {
    compare("top_flow_equal", "1,1");
}

#[test]
fn singleton_top_flow_preserves_output() {
    compare("top_flow_singleton", "0.87");
}
