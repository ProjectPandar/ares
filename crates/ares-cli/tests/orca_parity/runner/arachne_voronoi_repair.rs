//! Public witness for the default segment constructor at SkeletalTrapezoidation.cpp:388.
use super::plater_archive;
use crate::runner::{native_api_cli::sha, native_stream_case};

const PROJECT: &[u8] =
    include_bytes!("../../../../../tests/parity/concentric-ironing-order/project.3mf");
const CONTROL: &[u8] = include_bytes!(
    "../../../../../tests/parity/validation/arachne-open-wall-order/C0-neck0.25-r0.3mf"
);
const SOURCE: (&str, &[&str]) = (
    "arachne-voronoi-repair",
    &[
        "SkeletalTrapezoidation.cpp:373-388",
        "Voronoi.cpp:28-76,254-366",
    ],
);

#[test]
fn arachne_voronoi_repair_single_wall_matches_native() {
    assert_eq!(
        sha(PROJECT),
        "7143117d16ff40c408a30ecaba3abf504aa22aa64cd777b399834ff1d464dc6c"
    );
    let project = plater_archive::rewrite_archive(PROJECT, |name, text| {
        (name == "Metadata/project_settings.config").then(|| {
            let top = r#""ironing_type": "top""#;
            let walls = r#""wall_loops": "2""#;
            assert_eq!(text.matches(top).count(), 1);
            assert_eq!(text.matches(walls).count(), 1);
            text.replacen(top, r#""ironing_type": "solid""#, 1)
                .replacen(walls, r#""wall_loops": "1""#, 1)
        })
    });
    native_stream_case::check(
        SOURCE,
        "single-wall",
        &project,
        &sha(&project),
        (false, |reference, _| {
            for setting in ["; ironing_type = solid", "; wall_loops = 1"] {
                assert!(reference.native.lines().any(|line| line == setting));
            }
        }),
    );
}

#[test]
fn arachne_voronoi_no_repair_control_matches_native() {
    native_stream_case::check(
        SOURCE,
        "unrotated-wall",
        CONTROL,
        "6c5c67976071ea79e663b8a4d428f3f51202b3fb9821a49fbe6b9da1f399a07d",
        (false, |reference, _| {
            assert!(
                reference
                    .native
                    .lines()
                    .any(|line| line == "; wall_generator = arachne")
            );
        }),
    );
}
