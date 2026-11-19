use crate::runner::{OrcaRunner, application};

const PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/type2-tower-entry-z/j1-two-tools.3mf");

fn entry(gcode: &[u8]) -> &str {
    let text = std::str::from_utf8(gcode).unwrap();
    let (_, tail) = text.split_once("; layer 7 at 1.32mm\n").unwrap();
    let (block, _) = tail.split_once("; CP TOOLCHANGE WIPE\n").unwrap();
    assert!(block.starts_with("T0\n") && block.contains("\nG1 Z1.32\n"));
    block
}

#[test]
fn in_place_type2_entry_retains_terminal_z() {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native slicing oracle");
    let project = runner
        .fixture_case("type2-tower-entry-z", PROJECT.to_vec())
        .unwrap();
    let native = runner.slice_selected_plate_case(&project, 1).unwrap();
    let ares = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            PROJECT,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let root = tempfile::Builder::new()
        .prefix("type2-entry-streams-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    std::fs::write(root.join("plate_1.gcode"), &native.reference).unwrap();
    std::fs::write(root.join("ares.api.gcode"), &ares).unwrap();
    assert_eq!(entry(&ares), entry(&native.reference), "Type2 gap entry");
    crate::golden::compare_ordered_bytes_generator_only(&native.reference, &ares).unwrap();
}
