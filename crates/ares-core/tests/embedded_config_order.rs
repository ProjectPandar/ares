use ares_core::{ArcFittingMode, load_project};

#[test]
fn sequential_legacy_arc_mode_is_enabled_before_substitution() {
    check(include_bytes!(
        "../../../tests/parity/spiral-lift-arc-enabled/sequential.3mf"
    ));
}

#[test]
fn stock_legacy_arc_mode_is_enabled_before_substitution() {
    check(include_bytes!(
        "../../../tests/parity/spiral-lift-arc-enabled/stock.3mf"
    ));
}

#[test]
fn fuzzy_legacy_arc_mode_is_enabled_before_substitution() {
    check(include_bytes!(
        "../../../tests/parity/spiral-lift-arc-enabled/fuzzy.3mf"
    ));
}

fn check(input: &[u8]) {
    let project = load_project(input).unwrap();
    assert_eq!(
        project.settings().process.gcode.enable_arc_fitting,
        ArcFittingMode::Enabled,
        "PrintConfig.cpp:9886-9891 runs before Config.cpp:680-695"
    );
}
