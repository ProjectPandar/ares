use crate::runner::native_stream_case;

const BOUNDARY: &[&str] = &[
    "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19",
    "Fill/FillConcentric.cpp:170-179",
    "Arachne/utils/ExtrusionLine.hpp:212-230",
    "Polyline.cpp:657-667",
];

fn compare(label: &str, project: &[u8], sha: &str) {
    native_stream_case::check(
        ("concentric_thick_polyline_rotation", BOUNDARY),
        label,
        project,
        sha,
        (false, |native, _| {
            assert!(native.native.lines().any(|line| matches!(
                line,
                "; FEATURE: Internal solid infill" | ";TYPE:Internal solid infill"
            )));
        }),
    );
}

#[test]
fn x1c_concentric_solid_preserves_unequal_width_loop_starts() {
    compare(
        "x1c",
        include_bytes!("../../../../tests/parity/concentric-thick-polyline-rotation/x1c.3mf"),
        "d6876a5f80ede1081723fb08edd7a8067effb4e28d3c404de25d0db789710808",
    );
}

#[test]
fn k1_concentric_solid_matches_native() {
    compare(
        "k1",
        include_bytes!("../../../../tests/parity/concentric-thick-polyline-rotation/k1.3mf"),
        "fb5e91973281fca3c8c2309cfaff22a11bf0c393e5c2ac9c7d8ab1e272088e99",
    );
}

#[test]
fn mk4_concentric_solid_matches_native() {
    compare(
        "mk4",
        include_bytes!("../../../../tests/parity/concentric-thick-polyline-rotation/mk4.3mf"),
        "80ea8000584a07d9e0bb35f8509509ada6eb8c6eb92815ff713aebee608460e6",
    );
}
