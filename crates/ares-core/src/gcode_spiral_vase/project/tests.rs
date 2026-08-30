use super::{ProjectSpiralVaseConfig, ProjectSpiralVaseLayer, ProjectSpiralVaseRunState};

fn config() -> ProjectSpiralVaseConfig {
    ProjectSpiralVaseConfig {
        enabled: true,
        smooth_xy: false,
        max_xy_smoothing: 0.8,
        starting_flow_ratio: 1.0,
        finishing_flow_ratio: 0.0,
        resolution: 0.01,
        relative_e: true,
    }
}

fn layer(enabled: bool, final_layer: bool, z: f64) -> ProjectSpiralVaseLayer {
    ProjectSpiralVaseLayer {
        start: 0,
        enabled,
        final_layer,
        z,
        height: 0.2,
    }
}

#[test]
fn project_layer_filter_skips_travel_and_ramps_print_z() {
    let mut state = ProjectSpiralVaseRunState::new(config());
    let mut bottom = b"G1 X0 Y0 Z.2\nG1 X10 Y0 E1\n".to_vec();
    state.process_layer(&mut bottom, layer(false, false, 0.2));
    assert_eq!(bottom, b"G1 X0 Y0 Z.2\nG1 X10 Y0 E1\n");

    let mut body =
        b"G1 X0 Y0 Z.4\nG1 X10 Y0 E1\nG1 X10 Y10 E1\nG1 X0 Y10 E1\nG1 X0 Y0 E1\n".to_vec();
    state.process_layer(&mut body, layer(true, false, 0.4));
    let body = String::from_utf8(body).unwrap();

    assert!(!body.contains("G1 X0 Y0 Z.4\n"));
    assert!(body.contains("G1 X10 Y0 E1 Z.25\n"));
    assert!(body.contains("G1 X10 Y10 E1 Z.3\n"));
    assert!(body.contains("G1 X0 Y10 E1 Z.35\n"));
    assert!(body.contains("G1 X0 Y0 E1 Z.4\n"));
}

#[test]
fn final_relative_e_layer_appends_finishing_taper() {
    let mut state = ProjectSpiralVaseRunState::new(config());
    let mut first = b"G1 X0 Y0 Z.2\nG1 X0 Y10 E1\n".to_vec();
    state.process_layer(&mut first, layer(true, false, 0.2));
    let mut final_gcode = b"G1 X0 Y0 Z.4\nG1 X10 Y0 E1\nG1 X10 Y10 E1\n".to_vec();
    state.process_layer(&mut final_gcode, layer(true, true, 0.4));
    let final_gcode = String::from_utf8(final_gcode).unwrap();

    assert!(final_gcode.contains("G1 X10 Y0 E1 Z.3\nG1 X10 Y10 E1 Z.4\n"));
    assert!(
        final_gcode.ends_with("G1 X10 Y0 E.5\nG1 X10 Y10 E0\n"),
        "{final_gcode:?}"
    );
}
