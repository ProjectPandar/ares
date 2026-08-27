use crate::load_project;

// Captured OrcaSlicer 2.4.2 CLI `--export-3mf` output. The exporter writes
// thumbnail relationships (`3mf.cpp _add_relationships_file_to_archive`)
// without embedding the referenced PNG parts; loading must tolerate that.
const ORCA_CLI_EXPORT: &[u8] = include_bytes!("../../../../../tests/parity/orca_cli_ender3.3mf");

#[test]
fn project_load_accepts_orca_cli_export_without_thumbnail_parts() {
    let project = load_project(ORCA_CLI_EXPORT)
        .expect("OrcaSlicer CLI export without embedded previews must load");
    assert_eq!(project.plates().len(), 1);
}
