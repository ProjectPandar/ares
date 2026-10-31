use super::*;
use serde_json::json;
#[path = "first_layer_sequence_context.rs"]
mod context;
#[path = "first_layer_sequence_observation.rs"]
mod observation;

#[test]
fn g4_first_layer_sequence_orders_actual_wall_tools() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = parity::artifacts::root_from_env().unwrap();
    let mut proofs = Vec::new();
    let mut failures = Vec::new();
    for (label, sequence) in [
        ("empty", &[][..]),
        ("short", &[1][..]),
        ("forward", &[1, 2][..]),
        ("reverse", &[2, 1][..]),
        ("duplicate", &[2, 1, 2][..]),
    ] {
        let label = format!("first-layer-sequence/{label}");
        let exported = context::prepare(&runner, &label, sequence);
        let sliced = runner.slice_case(&exported).unwrap();
        let native = observation::observe(std::str::from_utf8(&sliced.reference).unwrap());
        let outcome = parity::compare_case(&sliced);
        let ares = std::fs::read(outcome.artifacts.as_ref().unwrap().join("ares.gcode"))
            .ok()
            .map(|bytes| observation::observe(std::str::from_utf8(&bytes).unwrap()));
        proofs.push(json!({"label":label,"sequence":sequence,"native":native,"ares":ares,"status":outcome.status,"detail":outcome.detail,"artifacts":outcome.artifacts}));
        parity::artifacts::overwrite(
            &root.join("first-layer-sequence-cases.json"),
            &serde_json::to_vec_pretty(&proofs).unwrap(),
        )
        .unwrap();
        observation::verify_first(&native);
        if let Some(ares) = &ares {
            observation::verify_first(ares);
        }
        if !sequence.is_empty() && sequence.len() >= 2 {
            let expected = if sequence[0] == 1 {
                json!([0, 1])
            } else {
                json!([1, 0])
            };
            assert_eq!(
                native["ordered_extrusion_runs"]["1"], expected,
                "Native active order"
            );
            if let Some(ares) = &ares {
                assert_eq!(
                    ares["ordered_extrusion_runs"]["1"], expected,
                    "Ares active order"
                );
            }
        }
        if outcome.status != "PASS" {
            failures.push(format!("{label}: {}", outcome.detail));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
