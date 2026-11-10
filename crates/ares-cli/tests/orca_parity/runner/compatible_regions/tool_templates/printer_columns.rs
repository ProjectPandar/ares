//! Extruder variant columns a two-extruder project leaves to its system printer
//! (`docs/architecture/system-printer-normalize-spec.md`).

use super::fixture::{Overrides, configured_project};
use std::fs;

#[test]
fn project_without_variant_columns_matches_native_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (project, reference, actual) = configured_project("", "", false, Overrides::default());
    let settings = crate::runner::application::settings(&project).unwrap();
    for key in ["extruder_variant_list", "default_nozzle_volume_type"] {
        assert!(
            !settings.contains_key(key),
            "the project must leave `{key}` to the system printer"
        );
    }
    let native = std::str::from_utf8(&reference).unwrap();
    let nozzles = native
        .lines()
        .find_map(|line| line.strip_prefix("; nozzle_diameter = "))
        .unwrap();
    assert!(
        nozzles.split(',').count() > 1,
        "Native must slice for several nozzles"
    );

    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("printer-columns-")
        .tempdir_in(&root)
        .unwrap()
        .keep();
    fs::write(directory.join("input.3mf"), &project).unwrap();
    fs::write(directory.join("ares.gcode"), &actual).unwrap();
    let cli = crate::runner::application::cli_command()
        .arg("slice")
        .arg("-o")
        .arg(directory.join("cli.gcode"))
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    assert!(cli.status.success(), "CLI slice: {directory:?}");
    assert_eq!(
        actual,
        fs::read(directory.join("cli.gcode")).unwrap(),
        "API/CLI raw output: {directory:?}"
    );

    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual)
        .unwrap_or_else(|difference| panic!("{difference}: {directory:?}"));
}
