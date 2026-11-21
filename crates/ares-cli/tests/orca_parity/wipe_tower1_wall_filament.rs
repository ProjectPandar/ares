use super::{PROJECT, rewrite_archive};
use crate::runner::{native_api_cli::sha, native_stream_case, repo_root};

fn one_layer_wall_filament(label: &str, height: &str, finishes_after_change: bool) {
    let source =
        std::fs::read(repo_root().join("tests/parity/bbl-type1-tower/enabled.3mf")).unwrap();
    let project = rewrite_archive(&source, |name, text| match name {
        "3D/3dmodel.model" => {
            // Scale each centered 10 mm cube to 0.2 mm in Z and leave its base at Z=0.
            assert_eq!(text.matches(" 0 0 1 128 ").count(), 2);
            assert_eq!(text.matches(" 5\" printable=").count(), 2);
            Some(
                text.replace(" 0 0 1 128 ", " 0 0 0.02 128 ")
                    .replace(" 5\" printable=", " 0.1\" printable="),
            )
        }
        PROJECT => {
            let mut settings: serde_json::Value = serde_json::from_str(text).unwrap();
            assert_eq!(
                settings["extruder_printable_height"],
                serde_json::json!(["0"])
            );
            settings["extruder_printable_height"] = serde_json::json!([height]);
            Some(serde_json::to_string(&settings).unwrap())
        }
        _ => None,
    });
    native_stream_case::check(
        (
            "bbl-type1-tower",
            &["GCode/WipeTower.cpp:4973–5001,5032–5050,5528–5533"],
        ),
        label,
        &project,
        &sha(&project),
        (false, |reference, _| {
            let native = &reference.native;
            assert_eq!(native.matches("; CHANGE_LAYER\n").count(), 1);
            assert!(native.contains("; Z_HEIGHT: 0.2\n"));
            let (before, after) = native.split_once("; CP TOOLCHANGE START\n").unwrap();
            assert!(before.contains("\nT1000\n"));
            assert!(after.contains("\nT1\n"));
            assert_eq!(
                before.matches("; FEATURE: Prime tower\n").count(),
                if finishes_after_change { 1 } else { 2 }
            );
            let (_, tail) = after.split_once("; CP TOOLCHANGE END\n").unwrap();
            assert_eq!(
                tail.contains("; FEATURE: Prime tower\n"),
                finishes_after_change
            );
        }),
    );
}

#[test]
fn type1_wall_uses_incoming_filament_above_last_change_height() {
    one_layer_wall_filament("wall-above-last-change-height", "0", true);
}

#[test]
fn type1_wall_keeps_outgoing_filament_within_last_change_height() {
    one_layer_wall_filament("wall-within-last-change-height", "256", false);
}
