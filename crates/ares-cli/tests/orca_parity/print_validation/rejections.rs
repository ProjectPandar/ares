//! Both sides must reject the same O1 input; oracle-only rejection is not parity.
use std::fs;

use ares_core::{ConfigOverrides, SliceError};
use serde_json::json;

use crate::{
    ParityOutcome,
    runner::{
        ExportedCase,
        stages::{FailureKind, Stage, StageError},
    },
};

pub(crate) fn compare(exported: &ExportedCase, error: &StageError) -> Option<ParityOutcome> {
    if error.stage != Stage::Slice || error.kind != FailureKind::Process {
        return None;
    }
    let captured = error.captured.as_ref()?;
    if captured.code != Some(if cfg!(unix) { 205 } else { -51 }) {
        return None;
    }
    let stderr = std::str::from_utf8(&captured.stderr).ok()?;
    let message = stderr.lines().next()?;
    let known = [
        "Relative extruder addressing requires resetting the extruder position at each layer to prevent loss of floating point accuracy. Add \"G92 E0\" to layer_gcode.",
        "\"G92 E0\" was found in before_layer_change_gcode, which is incompatible with absolute extruder addressing.",
        "\"G92 E0\" was found in layer_change_gcode, which is incompatible with absolute extruder addressing.",
        "\"G92 E0\" was found in before_layer_change_gcode, but the G or E are not uppercase. Please change them to the exact uppercase \"G92 E0\".",
        "\"G92 E0\" was found in layer_change_gcode, but the G or E are not uppercase. Please change them to the exact uppercase \"G92 E0\".",
    ];
    if !known.contains(&message) || stderr.lines().count() != 1 {
        return None;
    }
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("o1-rejection-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    fs::write(directory.join("input.3mf"), &exported.project).unwrap();
    let actual =
        ares_core::validate_project_plate(&exported.project, None, &ConfigOverrides::default());
    let matched =
        matches!(&actual, Err(SliceError::Validate { message: text, .. }) if text == message);
    let ares = match actual {
        Ok(()) => json!({"status": "VALIDATED"}),
        Err(error) => json!({"status":"ERROR", "error":error.to_string()}),
    };
    let report = json!({
        "printer":exported.label, "input_sha256":super::sha256(&exported.project),
        "status":if matched {"INPUT_REJECTED"} else {"DIVERGENT_REJECTION"},
        "oracle":{"code":captured.code,"error":message,"evidence":error.evidence},
        "ares":ares,
    });
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    Some(ParityOutcome {
        label: exported.label.clone(),
        status: if matched {
            "INPUT_REJECTED"
        } else {
            "DIVERGENT_REJECTION"
        },
        detail: "O1: compared both validation outcomes and exact error text".into(),
        artifacts: Some(directory),
    })
}
