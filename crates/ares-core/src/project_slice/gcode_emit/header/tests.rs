#[test]
fn automatic_role_widths_match_upstream_defaults() {
    let auto = crate::FloatOrPercent::Float(0.0);

    // `Flow::auto_extrusion_width` multiplies in FLOAT: 1.125f × 0.4f
    // = 0.45000002, not the f64 0.45.
    assert_eq!(super::resolved_width(auto, auto, 0.4, 1.125), 0.45000002);
    assert_eq!(super::resolved_width(auto, auto, 0.4, 1.0), 0.4);
    assert_eq!(super::width(auto, 0.4), None);
}

#[test]
fn header_values_trim_fractional_trailing_zeroes() {
    let values = [
        crate::OrcaFloat(1.3),
        crate::OrcaFloat(1.24),
        crate::OrcaFloat("1.12838".parse().unwrap()),
        crate::OrcaFloat(1.0),
    ];

    assert_eq!(super::format_values(&values), "1.3,1.24,1.12838,1");
}
