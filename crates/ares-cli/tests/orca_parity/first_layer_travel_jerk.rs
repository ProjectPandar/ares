//! `GCode.cpp:10519-10531`: first-layer travel jerk selection and positive gates.

fn layer_jerks(gcode: &str, index: usize) -> Vec<&str> {
    gcode
        .split(";LAYER_CHANGE\n")
        .nth(index + 1)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with("M205 "))
        .collect()
}

#[test]
fn first_layer_travel_jerk_percent_of_travel() {
    crate::runner::native_stream_case::check(
        (
            "first-layer-travel-jerk",
            &[
                "GCode.cpp:10519-10531::travel_to",
                "PrintConfig.cpp:4039-4047::initial_layer_travel_jerk",
                "Config.cpp:715-738::get_abs_value_at",
            ],
        ),
        "percent-half",
        include_bytes!("../../../../tests/parity/first-layer-travel-jerk/percent-half.3mf"),
        "240b0fe31d1cae446d6b79b43ee3705643ef29d27ae7818877773f19417b1654",
        (false, |reference, _| {
            let first = layer_jerks(&reference.native, 0);
            assert!(first.contains(&"M205 X6 Y6"));
            assert!(!first.contains(&"M205 X4 Y4"));
            assert!(!first.contains(&"M205 X12 Y12"));
            assert!(layer_jerks(&reference.native, 1).contains(&"M205 X12 Y12"));
        }),
    );
}

#[test]
fn first_layer_travel_jerk_uses_absolute_value() {
    crate::runner::native_stream_case::check(
        (
            "first-layer-travel-jerk",
            &["GCode.cpp:10519-10531::travel_to"],
        ),
        "absolute-three",
        include_bytes!("../../../../tests/parity/first-layer-travel-jerk/absolute-three.3mf"),
        "f4fa248165d43c748933d03d6d43a8773873ce61b47bd0745a050753ac3b6573",
        (false, |reference, _| {
            let first = layer_jerks(&reference.native, 0);
            assert!(first.contains(&"M205 X3 Y3"));
            assert!(!first.contains(&"M205 X12 Y12"));
            assert!(layer_jerks(&reference.native, 1).contains(&"M205 X12 Y12"));
        }),
    );
}

#[test]
fn zero_first_layer_travel_jerk_keeps_cached_extrusion_jerk() {
    crate::runner::native_stream_case::check(
        (
            "first-layer-travel-jerk",
            &[
                "GCode.cpp:10524-10531::travel_to",
                "GCodeWriter.cpp:483-489::set_jerk_xy",
            ],
        ),
        "zero-initial",
        include_bytes!("../../../../tests/parity/first-layer-travel-jerk/zero-initial.3mf"),
        "2ddd747fedf0c511a1cc3437c3e4594db86f5a25d96f886b48052dfb49ad379d",
        (false, |reference, _| {
            let first = layer_jerks(&reference.native, 0);
            assert!(!first.is_empty());
            assert!(first.iter().all(|line| *line == "M205 X8 Y8"));
            assert!(layer_jerks(&reference.native, 1).contains(&"M205 X12 Y12"));
        }),
    );
}

#[test]
fn first_layer_travel_jerk_defaults_to_full_travel_jerk() {
    crate::runner::native_stream_case::check(
        (
            "first-layer-travel-jerk",
            &[
                "GCode.cpp:10519-10531::travel_to",
                "PrintConfig.cpp:4039-4047::initial_layer_travel_jerk",
            ],
        ),
        "regular-control",
        include_bytes!("../../../../tests/parity/first-layer-travel-jerk/regular-control.3mf"),
        "4abbd244d5d70ed4d1b9a9814189c2e7cd3a8dfb069d8c76f3809c13c199b739",
        (false, |reference, _| {
            assert!(layer_jerks(&reference.native, 0).contains(&"M205 X12 Y12"));
            assert!(layer_jerks(&reference.native, 1).contains(&"M205 X12 Y12"));
        }),
    );
}

#[test]
fn zero_default_jerk_disables_first_layer_travel_jerk() {
    crate::runner::native_stream_case::check(
        (
            "first-layer-travel-jerk",
            &["GCode.cpp:10529-10531::travel_to"],
        ),
        "zero-default",
        include_bytes!("../../../../tests/parity/first-layer-travel-jerk/zero-default.3mf"),
        "0cd38da5e79a8fe2098a4eeed2a7fe1d92ffa8c1f451ad664a1f4fbd52f470e9",
        (false, |reference, _| {
            assert!(layer_jerks(&reference.native, 0).is_empty());
            assert!(layer_jerks(&reference.native, 1).is_empty());
        }),
    );
}
