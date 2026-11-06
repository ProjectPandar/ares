//! PrintConfig.cpp:10127-10146 and GCode.cpp:9277-9293: every acceleration
//! and jerk option in print_options_with_variant must retain its selected
//! variant vector before motion emission.

use crate::runner::application;

const INPUT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/parity/acceleration-jerk-variants/endpoint-contact.3mf"
));
const INPUT_SHA256: &str = "b1c6d193f0b990473cc857fdbde50a4ddfc7a739aff30bf3ce3810529a0e9529";

fn line_for_key(output: &str, key: &str) -> Option<String> {
    let prefix = format!("; {key} = ");
    let lines = output
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 1, "expected one config line for {key}");
    lines.into_iter().next()
}

fn assert_key_matches_native(key: &str) {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(crate::runner::native_api_cli::sha(INPUT), INPUT_SHA256);
    let case = crate::runner::native_api_cli::NativeCase::slice(
        &runner,
        INPUT,
        &format!("acceleration-jerk-variant-{key}-"),
    );
    let native_line = line_for_key(&case.native, key).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            INPUT,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let api_line = line_for_key(std::str::from_utf8(&api).unwrap(), key);
    assert_eq!(
        api_line,
        Some(native_line.clone()),
        "{key}: API config line differs from Native: native={native_line:?} api={api_line:?}"
    );
    let cli_path = case.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(case.root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(cli.status.success(), "{key}: CLI failed");
    let cli_bytes = std::fs::read(cli_path).unwrap();
    let cli_line = line_for_key(std::str::from_utf8(&cli_bytes).unwrap(), key);
    assert_eq!(
        cli_line,
        Some(native_line.clone()),
        "{key}: CLI config line differs from Native: native={native_line:?} cli={cli_line:?}"
    );
}

macro_rules! key_test {
    ($name:ident, $key:literal) => {
        #[test]
        fn $name() {
            assert_key_matches_native($key);
        }
    };
}

key_test!(default_acceleration_matches_native, "default_acceleration");
key_test!(bridge_acceleration_matches_native, "bridge_acceleration");
key_test!(travel_acceleration_matches_native, "travel_acceleration");
key_test!(
    initial_layer_travel_acceleration_matches_native,
    "initial_layer_travel_acceleration"
);
key_test!(
    initial_layer_acceleration_matches_native,
    "initial_layer_acceleration"
);
key_test!(
    outer_wall_acceleration_matches_native,
    "outer_wall_acceleration"
);
key_test!(
    inner_wall_acceleration_matches_native,
    "inner_wall_acceleration"
);
key_test!(
    sparse_infill_acceleration_matches_native,
    "sparse_infill_acceleration"
);
key_test!(
    internal_solid_infill_acceleration_matches_native,
    "internal_solid_infill_acceleration"
);
key_test!(
    top_surface_acceleration_matches_native,
    "top_surface_acceleration"
);
key_test!(default_jerk_matches_native, "default_jerk");
key_test!(outer_wall_jerk_matches_native, "outer_wall_jerk");
key_test!(inner_wall_jerk_matches_native, "inner_wall_jerk");
key_test!(infill_jerk_matches_native, "infill_jerk");
key_test!(top_surface_jerk_matches_native, "top_surface_jerk");
key_test!(initial_layer_jerk_matches_native, "initial_layer_jerk");
key_test!(travel_jerk_matches_native, "travel_jerk");
