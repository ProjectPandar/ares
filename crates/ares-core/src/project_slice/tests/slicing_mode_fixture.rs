use crate::{ProcessSlicingMode, SliceError, mesh_slicer::SlicingMode};

use super::{
    super::{
        chained_intersections::chain_project_intersections,
        looped_intersections::loop_project_intersections,
        slicing_mode_intersections::apply_project_slicing_modes, state::prepare_project_slice,
    },
    support::{KsrArchive, ksr_project},
};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";
const MODEL_SETTINGS: &str = "Metadata/model_settings.config";
const PROCESS_REGULAR: &str = r#""slicing_mode": "regular""#;
const PROCESS_EVEN_ODD: &str = r#""slicing_mode": "even_odd""#;
const PROCESS_CLOSE_HOLES: &str = r#""slicing_mode": "close_holes""#;
const OBJECT_PART_ANCHOR: &str = concat!(
    "    <metadata key=\"extruder\" value=\"1\"/>\n",
    "    <part id=\"1\" subtype=\"normal_part\">",
);
const OBJECT_EVEN_ODD_OVERRIDE: &str = concat!(
    "    <metadata key=\"extruder\" value=\"1\"/>\n",
    "    <metadata key=\"slicing_mode\" value=\"even_odd\"/>\n",
    "    <part id=\"1\" subtype=\"normal_part\">",
);
const LAYER_COUNT: usize = 460;

struct FixtureSnapshot {
    resolved_mode: ProcessSlicingMode,
    spiral_mode: bool,
    bottom_shell_layers: i32,
    bottom_shell_thickness: f64,
    modes: Vec<SlicingMode>,
}

#[test]
fn ksr_fixture_projects_regular_mode_from_3mf() {
    let snapshot = fixture_snapshot(ksr_project()).unwrap();

    assert_eq!(snapshot.resolved_mode, ProcessSlicingMode::Regular);
    assert!(!snapshot.spiral_mode);
    assert_eq!(snapshot.bottom_shell_layers, 3);
    assert_eq!(snapshot.bottom_shell_thickness, 0.0);
    assert_eq!(snapshot.modes, vec![SlicingMode::Regular; LAYER_COUNT]);
}

#[test]
fn process_slicing_modes_come_from_3mf_options() {
    let even_odd = fixture_snapshot(process_mode(PROCESS_EVEN_ODD)).unwrap();
    assert_eq!(even_odd.resolved_mode, ProcessSlicingMode::EvenOdd);
    assert_eq!(even_odd.modes, vec![SlicingMode::EvenOdd; LAYER_COUNT]);

    let close_holes = fixture_snapshot(process_mode(PROCESS_CLOSE_HOLES)).unwrap();
    assert_eq!(close_holes.resolved_mode, ProcessSlicingMode::CloseHoles);
    assert_eq!(close_holes.modes, vec![SlicingMode::Positive; LAYER_COUNT]);
}

#[test]
fn object_slicing_mode_override_wins_over_process_base() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(PROJECT_SETTINGS, PROCESS_REGULAR, PROCESS_CLOSE_HOLES);
    archive.replace_unique(MODEL_SETTINGS, OBJECT_PART_ANCHOR, OBJECT_EVEN_ODD_OVERRIDE);

    let overridden = fixture_snapshot(archive.bytes()).unwrap();

    assert_eq!(overridden.resolved_mode, ProcessSlicingMode::EvenOdd);
    assert_eq!(overridden.modes, vec![SlicingMode::EvenOdd; LAYER_COUNT]);
}

fn process_mode(mode: &str) -> Vec<u8> {
    let mut archive = KsrArchive::new();
    archive.replace_unique(PROJECT_SETTINGS, PROCESS_REGULAR, mode);
    archive.bytes()
}

fn fixture_snapshot(project: impl AsRef<[u8]>) -> Result<FixtureSnapshot, SliceError> {
    let state = prepare_project_slice(project)?;
    let resolved = state.resolved;
    let object = &resolved.objects[0];
    let region = &object.layer_candidates[0].model_parts[0].region;
    let resolved_mode = object.object.slicing_mode;
    let spiral_mode = resolved.views.full.process.print.spiral_mode.0;
    let bottom_shell_layers = region.bottom_shell_layers.0;
    let bottom_shell_thickness = region.bottom_shell_thickness.0;
    let max_gap_scaled = state.scale.checked_scale(2.0).unwrap();
    let chained = chain_project_intersections(state.intersected_objects);
    let looped = loop_project_intersections(chained, max_gap_scaled);
    let projected = apply_project_slicing_modes(looped, &resolved.objects, spiral_mode)?;
    let modes = projected
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flat_map(|volume| volume.into_parts().3)
        .map(|layer| layer.into_parts().0)
        .collect();
    Ok(FixtureSnapshot {
        resolved_mode,
        spiral_mode,
        bottom_shell_layers,
        bottom_shell_thickness,
        modes,
    })
}
