//! Public exported-project/output regression for `AdaptivePAProcessor`.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::{Map, Value, json};

#[path = "adaptive_pressure_advance/bridge_domain.rs"]
mod bridge_domain;
#[path = "adaptive_pressure_advance/residual_witness.rs"]
mod residual_witness;
#[path = "adaptive_pressure_advance/scarf_ramp.rs"]
mod scarf_ramp;
#[path = "adaptive_pressure_advance/sloped_loop.rs"]
mod sloped_loop;
#[path = "adaptive_pressure_advance/tools.rs"]
mod tools;

#[test]
fn adaptive_pa_constant_model() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let (machine, process, mut filaments) = settings();
    let settings = [
        ("enable_pressure_advance", json!(["1"])),
        ("pressure_advance", json!(["0.035"])),
        ("adaptive_pressure_advance", json!(["1"])),
        (
            "adaptive_pressure_advance_model",
            json!(["0.045,1,500\n0.045,20,500"]),
        ),
    ];
    for filament in &mut filaments {
        for (key, value) in &settings {
            filament.insert((*key).into(), value.clone());
        }
    }
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "adaptive-pa/constant-no-tower",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    for (key, value) in settings {
        runner::application::verify_value(&exported.project, key, &value).unwrap();
    }
    runner::application::verify_value(&exported.project, "enable_prime_tower", &json!("0"))
        .unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    let reference = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(!reference.contains(";TYPE:Prime tower\n"));
    assert_eq!(
        reference
            .lines()
            .filter(|line| line.starts_with("M900 K"))
            .collect::<Vec<_>>(),
        [
            "M900 K0.035; Override pressure advance value",
            "M900 K0.045; Override pressure advance value",
        ]
    );
    let outcome = crate::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

type PresetSettings = (
    Map<String, Value>,
    Map<String, Value>,
    Vec<Map<String, Value>>,
);

fn settings() -> PresetSettings {
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    process.insert("enable_prime_tower".into(), json!("0"));
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    (machine, process, filaments)
}

#[test]
fn adaptive_pa_models_and_activation() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let nonconstant = "0.015,1,100\n0.045,5,100\n0.09,20,100\n0.02,1,1000\n0.08,5,1000\n0.12,20,1000\n0.035,1,5000\n0.1,5,5000\n0.2,20,5000";
    let mut failures = Vec::new();
    for (label, model) in [
        ("interpolation", nonconstant),
        (
            "numeric-prefix",
            "+4.5e-2suffix,1flow,500accel\n+4.5e-2suffix,20flow,500accel",
        ),
        ("invalid", "not-a-model"),
        ("insufficient", "0.045,1,500"),
        ("negative-default", "not-a-model"),
        (
            "hexadecimal",
            "0x1p-5,0x1p0,0x1.f4p8\n0x1p-5,0x1.4p4,0x1.f4p8",
        ),
        ("integer-model", "10,1,500\n10,20,500"),
        ("scientific-model", "100000,1,500\n100000,20,500"),
        ("disabled-pa", nonconstant),
        ("disabled-adaptive", nonconstant),
        ("no-cooling", nonconstant),
        ("overhang", nonconstant),
        ("comments", nonconstant),
    ] {
        let (machine, mut process, mut filaments) = settings();
        let pa_enabled = label != "disabled-pa";
        let adaptive_enabled = label != "disabled-adaptive";
        let overhang = label == "overhang";
        let comments = label == "comments";
        let mut requested = vec![
            (
                "enable_pressure_advance",
                json!([if pa_enabled { "1" } else { "0" }]),
            ),
            (
                "pressure_advance",
                json!([if label == "negative-default" {
                    "-1"
                } else {
                    "0.035"
                }]),
            ),
            (
                "adaptive_pressure_advance",
                json!([if adaptive_enabled { "1" } else { "0" }]),
            ),
            ("adaptive_pressure_advance_model", json!([model])),
            (
                "adaptive_pressure_advance_overhangs",
                json!([if overhang { "1" } else { "0" }]),
            ),
            ("adaptive_pressure_advance_bridges", json!(["0.06"])),
            (
                "slow_down_for_layer_cooling",
                json!([if label == "no-cooling" { "0" } else { "1" }]),
            ),
        ];
        for filament in &mut filaments {
            for (key, value) in &requested {
                filament.insert((*key).into(), value.clone());
            }
        }
        for (key, value) in [
            ("gcode_comments", json!(if comments { "1" } else { "0" })),
            ("enable_overhang_speed", json!(["1"])),
        ] {
            process.insert(key.into(), value.clone());
            requested.push((key, value));
        }
        let input = if overhang {
            "overhang.stl"
        } else {
            "cube10.stl"
        };
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("adaptive-pa/{label}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &crate::smoke_overrides(),
                &runner::repo_root().join("tests/parity").join(input),
            )
            .unwrap();
        for (key, value) in requested {
            runner::application::verify_value(&exported.project, key, &value).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let commands: Vec<_> = reference
            .lines()
            .filter(|line| line.starts_with("M900 K"))
            .collect();
        match label {
            "hexadecimal" => {
                assert!(commands.contains(&"M900 K0.031; Override pressure advance value"))
            }
            "integer-model" => {
                assert!(commands.contains(&"M900 K10; Override pressure advance value"))
            }
            "scientific-model" => {
                assert!(commands.contains(&"M900 K1e+05; Override pressure advance value"))
            }
            "numeric-prefix" => {
                assert!(commands.contains(&"M900 K0.045; Override pressure advance value"))
            }
            "invalid" | "insufficient" | "disabled-adaptive" => assert!(
                commands
                    .iter()
                    .all(|line| *line == "M900 K0.035; Override pressure advance value")
            ),
            "disabled-pa" | "negative-default" => assert!(commands.is_empty()),
            _ => assert!(
                commands
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    > 3
            ),
        }
        if overhang {
            assert!(reference.contains(";TYPE:Bridge\n"));
            assert!(commands.contains(&"M900 K0.06; Override pressure advance value"));
        }
        if comments {
            assert!(reference.contains("; APA Speed Used: "));
        }
        let outcome = crate::compare_case(&sliced);
        if outcome.status != "PASS" {
            failures.push(format!("{label}: {}", outcome.detail));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
