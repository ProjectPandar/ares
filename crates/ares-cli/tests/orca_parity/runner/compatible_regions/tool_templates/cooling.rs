use super::fixture::{Overrides, configured};
use super::outputs;

#[test]
fn two_nozzle_vendor_indented_fan_commands_survive_output_processing() {
    if !crate::runner::reference_available() {
        return;
    }
    let (reference, actual) = outputs("", "", false, &[]);
    let commands = |bytes: &[u8]| {
        std::str::from_utf8(bytes)
            .unwrap()
            .lines()
            .filter(|line| line.starts_with(" M107 P") || line.starts_with("  M106 P"))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let expected = commands(&reference);
    assert!(expected.iter().any(|line| line == " M107 P0 ;fan off T0"));
    assert!(expected.iter().any(|line| line == " M107 P1 ;fan off T1"));
    assert_eq!(commands(&actual), expected);
}

#[test]
fn two_nozzle_first_layer_cooling_preserves_native_extrusion_feedrates() {
    if !crate::runner::reference_available() {
        return;
    }
    let (reference, actual) = outputs("", "", false, &[]);
    let feedrates = |bytes: &[u8]| {
        let text = std::str::from_utf8(bytes).unwrap();
        let layer = text.split(";LAYER_CHANGE\n").nth(1).unwrap();
        assert!(layer.lines().any(|line| line == "T1"));
        layer
            .lines()
            .filter(|line| line.starts_with("G1 F"))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let expected = feedrates(&reference);
    assert!(
        !expected.is_empty(),
        "Native first layer must emit extrusion feedrate commands"
    );
    assert_eq!(feedrates(&actual), expected);
}

#[test]
fn nonzero_initial_tool_uses_its_filament_cooling_settings() {
    if !crate::runner::reference_available() {
        return;
    }
    let (reference, actual) = configured(
        "",
        "",
        false,
        Overrides {
            process: &[
                ("outer_wall_filament_id", "2"),
                ("inner_wall_filament_id", "2"),
                ("internal_solid_filament_id", "1"),
                ("top_surface_filament_id", "1"),
                ("bottom_surface_filament_id", "2"),
                ("sparse_infill_filament_id", "1"),
                ("sparse_infill_density", "20%"),
            ],
            filaments: &[
                ("slow_down_for_layer_cooling", ["1", "1"]),
                ("slow_down_layer_time", ["5", "60"]),
                ("slow_down_min_speed", ["30", "5"]),
                ("dont_slow_down_outer_wall", ["0", "0"]),
            ],
            ..Default::default()
        },
    );
    let feedrates = |bytes: &[u8]| {
        let text = std::str::from_utf8(bytes).unwrap();
        assert!(text.lines().any(|line| line == "; filament: 1,2"));
        let (startup, body) = text.split_once(";LAYER_CHANGE\n").unwrap();
        assert_eq!(
            startup.lines().rfind(|line| matches!(*line, "T0" | "T1")),
            Some("T1")
        );
        let first_layer = body.split(";LAYER_CHANGE\n").next().unwrap();
        assert!(!first_layer.lines().any(|line| matches!(line, "T0" | "T1")));
        assert!(body.split("\nT0\n").skip(1).any(|later_tool| {
            later_tool
                .split("\nT1\n")
                .next()
                .unwrap()
                .contains(";TYPE:Sparse infill")
        }));
        first_layer
            .lines()
            .filter(|line| line.starts_with("G1 F"))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let expected = feedrates(&reference);
    assert!(expected.len() >= 3);
    assert_eq!(feedrates(&actual), expected);
}
