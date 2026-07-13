use serde_json::{Map, Value};

use crate::options::{
    FilamentOptions, Nullable, OrcaFloat, OrcaInt, Percent, ProcessOptions,
    ProjectPrintSourceOptions,
    region_options::{RegionBase, RegionOptionOverrides, RegionOverrideSources},
};

use super::{RegionOptions, inventory};

const FIXTURE: &[u8] =
    include_bytes!("../../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");

#[test]
fn fixture_resolves_region_only_from_typed_project_and_model_settings() {
    let project = crate::load_project(FIXTURE).unwrap();
    let raw: Value = serde_json::from_slice(project.project_settings_bytes()).unwrap();
    let raw = raw.as_object().unwrap();
    let rows = inventory();
    let process: ProcessOptions = serde_json::from_value(Value::Object(fields(
        raw,
        rows.iter()
            .filter(|row| row.raw_scope == "process")
            .map(|row| row.key.as_str()),
    )))
    .unwrap();
    let filament: FilamentOptions = serde_json::from_value(Value::Object(fields(
        raw,
        rows.iter()
            .filter(|row| row.raw_scope == "filament")
            .map(|row| row.key.as_str()),
    )))
    .unwrap();
    let project_print: ProjectPrintSourceOptions = serde_json::from_value(Value::Object(fields(
        raw,
        ProjectPrintSourceOptions::DECLARATION_ORDER,
    )))
    .unwrap();
    let num_extruders = project_print.nozzle_diameter.0.len();
    assert_eq!(num_extruders, 2);
    assert_eq!(
        project_print.nozzle_diameter.0,
        [OrcaFloat(0.4), OrcaFloat(0.4)]
    );
    let active_filament = active_filament_region(&filament.region, num_extruders);
    assert_eq!(active_filament.filament_ironing_flow.len(), num_extruders);
    assert_eq!(active_filament.filament_ironing_spacing.len(), num_extruders);
    assert_eq!(active_filament.filament_ironing_inset.len(), num_extruders);
    assert_eq!(active_filament.filament_ironing_speed.len(), num_extruders);

    let [object] = project.documents().model_settings.objects.as_slice() else {
        panic!("fixture must contain exactly one model-settings object");
    };
    let [part] = object.parts.as_slice() else {
        panic!("fixture must contain exactly one model-settings part");
    };
    assert_eq!(object.region_overrides.extruder, Some(OrcaInt(1)));
    assert_eq!(part.region_overrides, RegionOptionOverrides::default());
    assert_process_feature_ids_are_zero(&process);

    let effective = RegionOptions::resolve(
        &active_filament,
        RegionOverrideSources {
            base: RegionBase::ModelPart {
                process: &process.region,
                object: Some(&object.region_overrides),
                layer_range: None,
            },
            volume: &part.region_overrides,
            material: None,
        },
        num_extruders,
    );

    assert_effective_feature_ids_are_one(&effective);
    assert_eq!(effective.top_surface_filament_id, OrcaInt(1));
    assert_eq!(active_filament.filament_ironing_flow[0], Nullable::Nil);
    assert_eq!(active_filament.filament_ironing_spacing[0], Nullable::Nil);
    assert_eq!(active_filament.filament_ironing_inset[0], Nullable::Nil);
    assert_eq!(active_filament.filament_ironing_speed[0], Nullable::Nil);
    assert_eq!(effective.filament_ironing_flow, Percent(10.0));
    assert_eq!(effective.filament_ironing_spacing, OrcaFloat(0.15));
    assert_eq!(effective.filament_ironing_inset, OrcaFloat(0.21));
    assert_eq!(effective.filament_ironing_speed, OrcaFloat(30.0));
}

fn fields<'a>(raw: &Map<String, Value>, keys: impl IntoIterator<Item = &'a str>) -> Map<String, Value> {
    keys.into_iter()
        .map(|key| (key.to_owned(), raw[key].clone()))
        .collect()
}

fn active_filament_region(
    source: &crate::options::FilamentRegionSourceOptions,
    num_extruders: usize,
) -> crate::options::FilamentRegionSourceOptions {
    crate::options::FilamentRegionSourceOptions {
        filament_ironing_flow: active_values(&source.filament_ironing_flow, num_extruders),
        filament_ironing_spacing: active_values(&source.filament_ironing_spacing, num_extruders),
        filament_ironing_inset: active_values(&source.filament_ironing_inset, num_extruders),
        filament_ironing_speed: active_values(&source.filament_ironing_speed, num_extruders),
    }
}

fn active_values<T: Clone>(values: &[Nullable<T>], num_extruders: usize) -> Vec<Nullable<T>> {
    let variant_stride = values.len() / num_extruders;
    (0..num_extruders)
        .map(|index| values[index * variant_stride].clone())
        .collect()
}

fn assert_process_feature_ids_are_zero(process: &ProcessOptions) {
    assert_eq!(process.region.sparse_infill_filament_id, OrcaInt(0));
    assert_eq!(process.region.internal_solid_filament_id, OrcaInt(0));
    assert_eq!(process.region.top_surface_filament_id, OrcaInt(0));
    assert_eq!(process.region.bottom_surface_filament_id, OrcaInt(0));
    assert_eq!(process.region.outer_wall_filament_id, OrcaInt(0));
    assert_eq!(process.region.inner_wall_filament_id, OrcaInt(0));
}

fn assert_effective_feature_ids_are_one(effective: &RegionOptions) {
    assert_eq!(effective.sparse_infill_filament_id, OrcaInt(1));
    assert_eq!(effective.internal_solid_filament_id, OrcaInt(1));
    assert_eq!(effective.top_surface_filament_id, OrcaInt(1));
    assert_eq!(effective.bottom_surface_filament_id, OrcaInt(1));
    assert_eq!(effective.outer_wall_filament_id, OrcaInt(1));
    assert_eq!(effective.inner_wall_filament_id, OrcaInt(1));
}
