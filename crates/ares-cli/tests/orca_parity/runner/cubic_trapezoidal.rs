use super::super::native_stream_case;
use crate::golden;
use std::{fs, process::Command};

const SOURCE: &[&str] = &[
    "FillRectilinear.cpp:3603-3618 FillCubic count dispatch",
    "FillRectilinear.cpp:3057-3098 cubic_upper_level",
    "FillRectilinear.cpp:3379-3411 cubic trapezoidal rows",
];

macro_rules! complete_case {
    ($name:ident, $fixture:literal, $sha:literal) => {
        #[test]
        fn $name() {
            native_stream_case::check(
                ("cubic-trapezoidal", SOURCE),
                $fixture,
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/parity/cubic-trapezoidal/",
                    $fixture,
                    ".3mf"
                )),
                $sha,
                (false, |_, _| {}),
            );
        }
    };
}

complete_case!(
    cubic_count_two_matches_native_complete_output,
    "c2",
    "2d2ae9dfbe388a543ef1be150d38a7d7d68ba76e26cdd4dc0273c0095703ee6b"
);
complete_case!(
    cubic_count_three_matches_native_complete_output,
    "c3",
    "c90ee37eb3c4c953e9f95c953d6e7417020a594899aa482517f933f12972edbd"
);
complete_case!(
    cubic_holed_count_two_matches_native_complete_output,
    "hc2",
    "cc96edb57cc82b475baecb9352e7b4688dbe3c68fa50ee06d69332e494373511"
);

// This control explicitly asserts raw motion/progress, not complete-output parity.
// Its full comparator verdict remains recorded while the footer prerequisite is absent.
#[test]
fn cubic_count_one_matches_native_raw_motion_and_progress() {
    let project = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/parity/cubic-trapezoidal/c1.3mf"
    ));
    let (reference, _) = native_stream_case::qualify(
        ("cubic-trapezoidal", SOURCE),
        "c1",
        project,
        "054986b4c36ca0eebc76ecbf81481cc0b343e96a469ac6787c9ed250c4510141",
        false,
    );
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = reference.root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "CLI at {:?}", reference.root);
    let cli = fs::read(cli_path).unwrap();
    assert_eq!(api, cli, "count-one API/CLI byte equality");
    let records = |bytes: &[u8]| {
        bytes
            .split_inclusive(|byte| *byte == b'\n')
            .filter(|line| {
                [b"G0 ".as_slice(), b"G1 ", b"G2 ", b"G3 ", b"M73 "]
                    .iter()
                    .any(|prefix| line.starts_with(prefix))
            })
            .flatten()
            .copied()
            .collect::<Vec<_>>()
    };
    let native_records = records(reference.native.as_bytes());
    let api_records = records(&api);
    fs::write(
        reference.root.join("raw-control.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "assertion_boundary": "exact ordered raw G0/G1/G2/G3 plus M73; API==CLI",
            "raw_motion_progress_equal": native_records == api_records,
            "api_cli_equal": api == cli,
            "full_output_comparison": golden::compare_ordered_bytes_generator_only(
                reference.native.as_bytes(), &api
            ),
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(native_records, api_records, "count-one raw motion/progress");
}
