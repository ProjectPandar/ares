use crate::{self as parity, runner::OrcaRunner};
use serde_json::json;
use std::fs;

#[path = "timelapse_context_fixture.rs"]
mod fixture;
#[path = "timelapse_context_observation.rs"]
mod observation;

#[test]
fn timelapse_templates_render_current_logical_filament_vectors() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = parity::artifacts::root_from_env().unwrap();
    let mut proofs = Vec::new();
    let mut failures = Vec::new();
    let mut vendor_output = None;
    for (name, first, enabled, handoff, explicit) in [
        ("vendor", [2, 1], false, false, false),
        ("implicit-reverse", [2, 1], true, false, false),
        ("implicit-slot0", [1, 2], true, false, false),
        ("handoff", [2, 1], true, true, false),
        ("explicit", [2, 1], true, false, true),
        ("empty", [2, 1], false, false, false),
    ] {
        let label = format!("timelapse-template-context/{name}");
        let script = format!(
            "; TLCTX {{layer_num}} {{current_extruder}} {{layer_z}}\n{}\n; TLCTX_END",
            if explicit {
                "M104 S[nozzle_temperature_0]\nM104 S[nozzle_temperature_1]"
            } else {
                "M104 S[nozzle_temperature]"
            }
        );
        let source = if enabled {
            Some(script.as_str())
        } else if name == "empty" {
            Some("")
        } else {
            None
        };
        let exported = fixture::prepare(&runner, &label, &first, source, handoff);
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        let directory = outcome.artifacts.as_ref().unwrap();
        let native = observation::observe(std::str::from_utf8(&sliced.reference).unwrap());
        let actual = fs::read(directory.join("ares.gcode")).unwrap();
        let ares = observation::observe(std::str::from_utf8(&actual).unwrap());
        let cli = crate::runner::application::cli_command()
            .arg("slice")
            .arg("-o")
            .arg(directory.join("cli.gcode"))
            .arg(directory.join("input.3mf"))
            .output()
            .unwrap();
        parity::artifacts::write(&directory.join("cli.stdout"), &cli.stdout).unwrap();
        parity::artifacts::write(&directory.join("cli.stderr"), &cli.stderr).unwrap();
        parity::artifacts::write(
            &directory.join("cli.exit.json"),
            &serde_json::to_vec(&cli.status.code()).unwrap(),
        )
        .unwrap();
        let cli_bytes = fs::read(directory.join("cli.gcode")).unwrap();
        let cli_observed = observation::observe(std::str::from_utf8(&cli_bytes).unwrap());
        proofs.push(json!({"label":label,"native":native,"ares":ares,"cli":cli_observed,
            "cli_exit":cli.status.code(),"status":outcome.status,"detail":outcome.detail,"artifacts":directory}));
        parity::artifacts::overwrite(
            &root.join("timelapse-template-context-cases.json"),
            &serde_json::to_vec_pretty(&proofs).unwrap(),
        )
        .unwrap();
        assert!(cli.status.success(), "CLI admission: {directory:?}");
        assert_eq!(actual, cli_bytes, "API/CLI raw output");
        let cli_comparison =
            crate::golden::compare_ordered_bytes_generator_only(&sliced.reference, &cli_bytes);
        observation::verify(&native, enabled, handoff, explicit);
        if name == "vendor" {
            vendor_output = Some(actual.clone());
        }
        if name == "empty" {
            crate::golden::compare_ordered_bytes_generator_only(
                &sliced.reference,
                vendor_output.as_ref().unwrap(),
            )
            .unwrap();
        }
        if outcome.status == "PASS" {
            observation::verify(&ares, enabled, handoff, explicit);
            assert_eq!(native, ares, "Native/API hook observations");
        } else {
            failures.push(format!("{label}: {}", outcome.detail));
        }
        if let Err(error) = cli_comparison {
            failures.push(format!("{label} CLI: {error:?}"));
        } else {
            observation::verify(&cli_observed, enabled, handoff, explicit);
            assert_eq!(native, cli_observed, "Native/CLI hook observations");
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
