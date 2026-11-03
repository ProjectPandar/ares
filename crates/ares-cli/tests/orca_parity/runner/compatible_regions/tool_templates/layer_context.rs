use crate::{self as parity, runner::OrcaRunner};
use serde_json::json;
use std::fs;

#[path = "layer_context_fixture.rs"]
pub(super) mod fixture;
#[path = "layer_context_observation.rs"]
mod observation;

#[test]
fn layer_templates_render_current_logical_filament_vectors() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = parity::artifacts::root_from_env().unwrap();
    let mut proofs = Vec::new();
    let mut failures = Vec::new();
    for (name, first, before, after, handoff, explicit) in [
        ("vendor", [2, 1], false, false, false, false),
        ("before", [2, 1], true, false, false, false),
        ("after", [2, 1], false, true, false, false),
        ("both-slot0", [1, 2], true, true, false, false),
        ("handoff", [2, 1], true, true, true, false),
        ("explicit", [2, 1], true, true, false, true),
    ] {
        let label = format!("layer-template-context/{name}");
        let script = |hook| {
            format!(
                "; LCTX {hook} {{layer_num}} {{current_extruder}} {{layer_z}}\nM104 S{}\n; LCTX_INDEX [nozzle_temperature_0] [nozzle_temperature_1]",
                if explicit {
                    "[nozzle_temperature_0]"
                } else {
                    "[nozzle_temperature]"
                }
            )
        };
        let before_script = script("before");
        let after_script = script("after");
        let mut hooks = Vec::new();
        if before {
            hooks.push(("before_layer_change_gcode", before_script.as_str()));
        }
        if after {
            hooks.push(("layer_change_gcode", after_script.as_str()));
        }
        let exported = fixture::prepare(&runner, &label, &first, &hooks, handoff);
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
            &root.join("layer-template-context-cases.json"),
            &serde_json::to_vec_pretty(&proofs).unwrap(),
        )
        .unwrap();
        assert!(cli.status.success(), "CLI admission: {directory:?}");
        // Whole-output strict checks run before behavior assertions; failures
        // retain every case's Native/API/CLI streams and observations.
        let cli_comparison =
            crate::golden::compare_ordered_bytes_generator_only(&sliced.reference, &cli_bytes);
        observation::verify(&native, before || after, before && after, handoff, explicit);
        if outcome.status == "PASS" {
            observation::verify(&ares, before || after, before && after, handoff, explicit);
        } else {
            failures.push(format!("{label}: {}", outcome.detail));
        }
        if let Err(error) = cli_comparison {
            failures.push(format!("{label} CLI: {error:?}"));
        } else {
            observation::verify(
                &cli_observed,
                before || after,
                before && after,
                handoff,
                explicit,
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
