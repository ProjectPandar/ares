use super::{LEGS, OrcaRunner, native, project, sha};
use std::fs;

#[test]
fn twisted_aligned_seam_preserves_native_wipe_section() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let root = tempfile::Builder::new()
        .prefix("twisted-aligned-seam-wipe-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = project(&LEGS[1]);
    let reference = native(&runner, &root.join("case"), &project);
    let expected = wipe_section(&reference);
    assert!(expected.contains("G1 X122.165 Y120.533 E-.05164\n"));
    fs::write(root.join("native-wipe.gcode"), expected).unwrap();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let actual = runtime
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(root.join("ares.validated.gcode"), &actual).unwrap();
    let actual_section = wipe_section(&actual);
    fs::write(root.join("api-wipe.gcode"), actual_section).unwrap();
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "scope": "selected wipe block, not whole-output parity",
            "input_sha256": sha(&project),
            "native_sha256": sha(&reference),
            "api_sha256": sha(&actual),
            "native_wipe": expected,
            "api_wipe": actual_section,
            "section_equal": actual_section == expected,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(actual_section, expected, "{root:?}");
}

fn wipe_section(bytes: &[u8]) -> &str {
    let mut sections = std::str::from_utf8(bytes)
        .unwrap()
        .split(";WIPE_START\n")
        .skip(1)
        .map(|tail| tail.split_once(";WIPE_END\n").unwrap().0)
        .filter(|section| section.contains("G1 X122.165 Y120.533 E-"));
    let section = sections.next().unwrap();
    assert!(sections.next().is_none());
    section
}
