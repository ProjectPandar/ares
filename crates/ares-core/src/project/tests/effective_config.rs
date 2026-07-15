mod caller;
mod candidates;
mod cardinality;
mod fixture;
mod grouping;
mod layers;
mod occupancy;
mod phases;
mod selector_validation;
mod support;
mod usage;

use crate::{
    Nullable, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, OrcaPercents, Percent, Project,
    ProjectSettings, SliceError, load_project,
    project::effective_config::{ValidatedMaterializedProject, validate_materialized_project},
};

use support::ProjectParts;

fn valid_settings(physical_count: usize, logical_count: usize) -> ProjectSettings {
    assert!(physical_count > 0);
    assert!(logical_count > 0);

    let mut settings = ProjectSettings::default();
    settings.project.print.nozzle_diameter = OrcaFloats(
        (0..physical_count)
            .map(|index| OrcaFloat(0.4 + index as f64 * 0.1))
            .collect(),
    );
    settings.filament.gcode.filament_diameter = OrcaFloats(vec![OrcaFloat(1.75); logical_count]);
    settings.project.gcode.filament_map = OrcaInts(
        (0..logical_count)
            .map(|index| OrcaInt((index % physical_count + 1) as i32))
            .collect(),
    );
    settings.filament.region.filament_ironing_flow = vec![Nullable::Nil; logical_count];
    settings.filament.region.filament_ironing_spacing = vec![Nullable::Nil; logical_count];
    settings.filament.region.filament_ironing_inset = vec![Nullable::Nil; logical_count];
    settings.filament.region.filament_ironing_speed = vec![Nullable::Nil; logical_count];
    settings.filament.print.filament_shrink = OrcaPercents(vec![Percent(100.0); logical_count]);
    settings.filament.print.filament_shrinkage_compensation_z =
        OrcaPercents(vec![Percent(100.0); logical_count]);
    settings
}

fn valid_project() -> Project {
    load_project(ProjectParts::valid().bytes()).unwrap()
}

fn validate(
    settings: &ProjectSettings,
    project: &Project,
) -> Result<ValidatedMaterializedProject, SliceError> {
    validate_materialized_project(settings, project)
}

fn assert_invalid_key(result: Result<ValidatedMaterializedProject, SliceError>, key: &str) {
    assert_eq!(
        result.unwrap_err(),
        SliceError::InvalidInput(format!("invalid Orca option {key}"))
    );
}

fn assert_unsupported_key(result: Result<ValidatedMaterializedProject, SliceError>, key: &str) {
    assert_eq!(
        result.unwrap_err(),
        SliceError::UnsupportedProjectFeature(key.to_owned())
    );
}
