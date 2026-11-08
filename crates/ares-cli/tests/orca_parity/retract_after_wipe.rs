//! Native GCode.cpp:538–602,10691–10722; Extruder.cpp:202–213.
//! Failures: inactive clamp, footer-only activation, dropped nullable override,
//! wrong phase/cap, doubled final retract, API/CLI or ordered output mismatch.

use crate::runner::{OrcaRunner, native_api_cli::NativeCase};
use serde::Serialize;
use std::fs;

#[path = "retract_after_wipe/input.rs"]
mod input;

#[derive(Debug, Serialize)]
struct Phase {
    before: Option<f64>,
    during: f64,
    after: Option<f64>,
}

fn extrusion(line: &str) -> Option<f64> {
    if !line.starts_with("G1 ") {
        return None;
    }
    let value = line
        .split(';')
        .next()
        .unwrap()
        .split_whitespace()
        .find_map(|word| word.strip_prefix('E'))?
        .parse::<f64>()
        .unwrap();
    assert!(value.is_finite());
    Some(value)
}

fn phases(text: &str) -> Vec<Phase> {
    let lines: Vec<_> = text.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == ";WIPE_START")
        .map(|(start, _)| {
            let end = start
                + lines[start..]
                    .iter()
                    .position(|l| *l == ";WIPE_END")
                    .unwrap();
            let moves: Vec<_> = lines[start..end]
                .iter()
                .filter(|line| {
                    line.starts_with("G1 ") && line.contains(" X") && line.contains(" Y")
                })
                .collect();
            assert!(!moves.is_empty(), "Native lacks real XY wipe");
            let during = moves
                .iter()
                .filter_map(|line| extrusion(line))
                .inspect(|e| {
                    assert!(*e <= 0.0, "Native wipe retract direction");
                })
                .sum();
            Phase {
                before: extrusion(lines[start - 1]),
                during,
                after: extrusion(lines[end + 1]),
            }
        })
        .collect()
}

fn check(label: &str, after: &str, override_after: &str, wipe: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = input::project(after, override_after, if wipe { "1" } else { "0" });
    let case = NativeCase::slice(&runner, &project, &format!("retract-after-wipe-{label}-"));
    let actual = phases(&case.native);
    if wipe {
        assert!(!actual.is_empty(), "Native wipe activation {:?}", case.root);
    } else {
        assert!(
            actual.is_empty(),
            "Native wipe-disabled control {:?}",
            case.root
        );
    }
    let active = wipe && (after != "0%" || override_after != "nil");
    if active {
        let baseline = NativeCase::slice(
            &runner,
            &input::project("0%", "nil", "1"),
            &format!("retract-after-wipe-{label}-zero-"),
        );
        let zero = phases(&baseline.native);
        assert_eq!(actual.len(), zero.len(), "Native phase identity/count");
        let a = &actual[0];
        let b = &zero[0];
        assert!(a.before.unwrap() < 0.0, "Native before phase absent");
        assert_eq!(a.before, b.before, "Native must retain before reservation");
        assert!(
            a.during > b.during,
            "Native nonzero-after must reduce during retract"
        );
        assert!(
            a.after.unwrap() < b.after.unwrap_or(0.0),
            "Native after reservation absent"
        );
        if label == "clamped" {
            assert_eq!(a.during, 0.0, "Native after cap exhausts remaining budget");
        } else {
            assert!(a.during < 0.0, "Native positive remaining during budget");
        }
        fs::write(
            case.root.join("native-zero-premise.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "baseline_root":baseline.root,"baseline_phases":zero,
                "baseline_sha256":crate::runner::native_api_cli::sha(baseline.native.as_bytes())
            }))
            .unwrap(),
        )
        .unwrap();
    }
    fs::write(
        case.root.join("premise.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "label":label,"machine_after":after,"filament_after":override_after,
            "wipe":wipe,"active_after":active,"native_phases":actual,
            "source":"GCode.cpp:538-602,10691-10722; Extruder.cpp:202-213"
        }))
        .unwrap(),
    )
    .unwrap();
    let _ = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
}

#[test]
fn machine_after_reserves_retraction() {
    check("machine", "25%", "nil", true);
}

#[test]
fn after_is_clamped_to_one_minus_before() {
    check("clamped", "100%", "nil", true);
}

#[test]
fn filament_override_reserves_retraction() {
    check("filament", "0%", "25%", true);
}

#[test]
fn zero_after_control() {
    check("zero", "0%", "nil", true);
}

#[test]
fn disabled_wipe_control() {
    check("disabled", "25%", "nil", false);
}
