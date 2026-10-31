//! Logical-filament activation and PA resets at `GCode::set_extruder`.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::json;

#[test]
fn adaptive_pa_tool_changes() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Snapmaker",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Snapmaker", "Snapmaker J1 (0.4 nozzle)").unwrap();
    for adaptive in [["1", "1"], ["1", "0"]] {
        let mut machine = profiles.machine(&selection.printer).unwrap();
        machine.insert("filament_map_mode".into(), json!("Manual"));
        machine.insert("filament_map".into(), json!(["1", "2"]));
        let mut process = profiles.process(&selection.process).unwrap();
        crate::normalize_process_defaults(&machine, &mut process);
        for (key, value) in [
            ("enable_support", "0"),
            ("enable_prime_tower", "0"),
            ("sparse_infill_density", "0%"),
            ("sparse_infill_pattern", "rectilinear"),
            ("sparse_infill_filament_id", "2"),
            ("gcode_comments", "1"),
        ] {
            process.insert(key.into(), json!(value));
        }
        let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
        crate::normalize_filament_defaults(&mut filaments);
        let requested = [
            ("filament_colour", ["#F2754E", "#0000FF"]),
            ("enable_pressure_advance", ["1", "1"]),
            ("pressure_advance", ["0.035", "0.055"]),
            ("adaptive_pressure_advance", adaptive),
            (
                "adaptive_pressure_advance_model",
                ["0.045,1,500\n0.045,20,500", "0.08,1,500\n0.08,20,500"],
            ),
        ];
        for (key, values) in &requested {
            for (filament, value) in filaments.iter_mut().zip(values) {
                filament.insert((*key).into(), json!([value]));
            }
        }
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("adaptive-pa/tools-{}-{}", adaptive[0], adaptive[1]),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &crate::smoke_overrides(),
                &runner::repo_root().join("tests/parity/overhang.stl"),
            )
            .unwrap();
        let exported = runner
            .bind_filament_map_mode(exported, &json!("Manual"))
            .unwrap();
        for (key, values) in requested {
            runner::application::verify_value(&exported.project, key, &json!(values)).unwrap();
        }
        for (key, value) in [
            ("sparse_infill_filament_id", json!("2")),
            ("filament_map", json!(["1", "2"])),
            ("gcode_comments", json!("1")),
        ] {
            runner::application::verify_value(&exported.project, key, &value).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(
            reference
                .lines()
                .filter(|line| *line == "T0" || *line == "T1")
                .count()
                > 3
        );
        assert!(reference.contains("; PA_CHANGE:T0 "));
        assert_eq!(reference.contains("; PA_CHANGE:T1 "), adaptive[1] == "1");
        assert!(reference.contains("M900 K0.045; Override pressure advance value"));
        assert_eq!(
            reference.contains("M900 K0.08; Override pressure advance value"),
            adaptive[1] == "1"
        );
        let outcome = crate::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    }
}
