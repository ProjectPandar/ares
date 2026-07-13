use crate::options::{
    FilamentRegionSourceOptions, Nullable, OrcaFloat, OrcaInt, Percent,
    ProcessRegionSourceOptions,
    region_options::{RegionBase, RegionOptionOverrides, RegionOverrideSources},
};

use super::RegionOptions;

#[test]
fn final_top_surface_id_alone_selects_all_four_non_nil_filament_values() {
    let filament = filament(
        [Percent(11.0), Percent(21.0)],
        [OrcaFloat(12.0), OrcaFloat(22.0)],
        [OrcaFloat(13.0), OrcaFloat(23.0)],
        [OrcaFloat(14.0), OrcaFloat(24.0)],
    );
    let first = resolve_model_part(
        &filament,
        process_with_top_and_ordinary(1),
        &RegionOptionOverrides::default(),
    );
    let second = resolve_model_part(
        &filament,
        process_with_top_and_ordinary(2),
        &RegionOptionOverrides::default(),
    );

    assert_selected(&first, Percent(11.0), 12.0, 13.0, 14.0);
    assert_selected(&second, Percent(21.0), 22.0, 23.0, 24.0);
}

#[test]
fn nil_inherits_each_final_ordinary_value_and_mixed_entries_are_independent() {
    let process = process_with_top_and_ordinary(2);
    let all_nil = FilamentRegionSourceOptions {
        filament_ironing_flow: vec![Nullable::Nil, Nullable::Nil],
        filament_ironing_spacing: vec![Nullable::Nil, Nullable::Nil],
        filament_ironing_inset: vec![Nullable::Nil, Nullable::Nil],
        filament_ironing_speed: vec![Nullable::Nil, Nullable::Nil],
    };
    let inherited = resolve_model_part(&all_nil, process.clone(), &RegionOptionOverrides::default());
    assert_selected(&inherited, Percent(31.0), 32.0, 33.0, 34.0);

    let mixed = FilamentRegionSourceOptions {
        filament_ironing_flow: vec![Nullable::Value(Percent(101.0)), Nullable::Nil],
        filament_ironing_spacing: vec![Nullable::Value(OrcaFloat(102.0)), Nullable::Value(OrcaFloat(202.0))],
        filament_ironing_inset: vec![Nullable::Value(OrcaFloat(103.0)), Nullable::Nil],
        filament_ironing_speed: vec![Nullable::Value(OrcaFloat(104.0)), Nullable::Value(OrcaFloat(204.0))],
    };
    let selected = resolve_model_part(&mixed, process, &RegionOptionOverrides::default());
    assert_selected(&selected, Percent(31.0), 202.0, 33.0, 204.0);
}

#[test]
fn invalid_top_surface_id_is_clamped_before_filament_index_selection() {
    let filament = filament(
        [Percent(41.0), Percent(51.0)],
        [OrcaFloat(42.0), OrcaFloat(52.0)],
        [OrcaFloat(43.0), OrcaFloat(53.0)],
        [OrcaFloat(44.0), OrcaFloat(54.0)],
    );
    let actual = resolve_model_part(
        &filament,
        process_with_top_and_ordinary(9),
        &RegionOptionOverrides::default(),
    );

    assert_eq!(actual.top_surface_filament_id, OrcaInt(1));
    assert_selected(&actual, Percent(41.0), 42.0, 43.0, 44.0);
}

#[test]
fn modifier_discards_parent_selected_values_and_reselects_after_each_final_top_override() {
    let parent_filament = filament(
        [Percent(901.0), Percent(911.0)],
        [OrcaFloat(902.0), OrcaFloat(912.0)],
        [OrcaFloat(903.0), OrcaFloat(913.0)],
        [OrcaFloat(904.0), OrcaFloat(914.0)],
    );
    let parent = resolve_model_part(
        &parent_filament,
        process_with_top_and_ordinary(1),
        &RegionOptionOverrides::default(),
    );
    assert_selected(&parent, Percent(901.0), 902.0, 903.0, 904.0);

    let filament = filament(
        [Percent(101.0), Percent(201.0)],
        [OrcaFloat(102.0), OrcaFloat(202.0)],
        [OrcaFloat(103.0), OrcaFloat(203.0)],
        [OrcaFloat(104.0), OrcaFloat(204.0)],
    );
    let volume = RegionOptionOverrides {
        top_surface_filament_id: Some(OrcaInt(2)),
        ..Default::default()
    };
    let volume_selected = RegionOptions::resolve(
        &filament,
        RegionOverrideSources {
            base: RegionBase::Modifier { parent: &parent },
            volume: &volume,
            material: None,
        },
        2,
    );
    assert_selected(&volume_selected, Percent(201.0), 202.0, 203.0, 204.0);

    let material = RegionOptionOverrides {
        top_surface_filament_id: Some(OrcaInt(1)),
        ..Default::default()
    };
    let material_selected = RegionOptions::resolve(
        &filament,
        RegionOverrideSources {
            base: RegionBase::Modifier { parent: &parent },
            volume: &volume,
            material: Some(&material),
        },
        2,
    );
    assert_selected(&material_selected, Percent(101.0), 102.0, 103.0, 104.0);
}

#[test]
fn selected_nil_inherits_ordinary_values_from_the_final_scope_in_both_base_branches() {
    let nil = FilamentRegionSourceOptions {
        filament_ironing_flow: vec![Nullable::Nil, Nullable::Nil],
        filament_ironing_spacing: vec![Nullable::Nil, Nullable::Nil],
        filament_ironing_inset: vec![Nullable::Nil, Nullable::Nil],
        filament_ironing_speed: vec![Nullable::Nil, Nullable::Nil],
    };
    let process = process_with_top_and_ordinary(1);
    let layer_range = ordinary_overrides(41.0);
    let empty = RegionOptionOverrides::default();
    let model_part = RegionOptions::resolve(
        &nil,
        RegionOverrideSources {
            base: RegionBase::ModelPart {
                process: &process,
                object: None,
                layer_range: Some(&layer_range),
            },
            volume: &empty,
            material: None,
        },
        2,
    );
    assert_selected(&model_part, Percent(41.0), 42.0, 43.0, 44.0);

    let parent_filament = filament(
        [Percent(901.0), Percent(911.0)],
        [OrcaFloat(902.0), OrcaFloat(912.0)],
        [OrcaFloat(903.0), OrcaFloat(913.0)],
        [OrcaFloat(904.0), OrcaFloat(914.0)],
    );
    let parent = resolve_model_part(&parent_filament, process, &empty);
    let material = ordinary_overrides(51.0);
    let modifier = RegionOptions::resolve(
        &nil,
        RegionOverrideSources {
            base: RegionBase::Modifier { parent: &parent },
            volume: &empty,
            material: Some(&material),
        },
        2,
    );
    assert_selected(&modifier, Percent(51.0), 52.0, 53.0, 54.0);
}

fn process_with_top_and_ordinary(top_surface_filament_id: i32) -> ProcessRegionSourceOptions {
    ProcessRegionSourceOptions {
        top_surface_filament_id: OrcaInt(top_surface_filament_id),
        ironing_flow: Percent(31.0),
        ironing_spacing: OrcaFloat(32.0),
        ironing_inset: OrcaFloat(33.0),
        ironing_speed: OrcaFloat(34.0),
        ..Default::default()
    }
}

fn filament(
    flow: [Percent; 2],
    spacing: [OrcaFloat; 2],
    inset: [OrcaFloat; 2],
    speed: [OrcaFloat; 2],
) -> FilamentRegionSourceOptions {
    FilamentRegionSourceOptions {
        filament_ironing_flow: flow.into_iter().map(Nullable::Value).collect(),
        filament_ironing_spacing: spacing.into_iter().map(Nullable::Value).collect(),
        filament_ironing_inset: inset.into_iter().map(Nullable::Value).collect(),
        filament_ironing_speed: speed.into_iter().map(Nullable::Value).collect(),
    }
}

fn ordinary_overrides(base: f64) -> RegionOptionOverrides {
    RegionOptionOverrides {
        ironing_flow: Some(Percent(base)),
        ironing_spacing: Some(OrcaFloat(base + 1.0)),
        ironing_inset: Some(OrcaFloat(base + 2.0)),
        ironing_speed: Some(OrcaFloat(base + 3.0)),
        ..Default::default()
    }
}

fn resolve_model_part(
    filament: &FilamentRegionSourceOptions,
    process: ProcessRegionSourceOptions,
    volume: &RegionOptionOverrides,
) -> RegionOptions {
    RegionOptions::resolve(
        filament,
        RegionOverrideSources {
            base: RegionBase::ModelPart {
                process: &process,
                object: None,
                layer_range: None,
            },
            volume,
            material: None,
        },
        2,
    )
}

fn assert_selected(
    actual: &RegionOptions,
    flow: Percent,
    spacing: f64,
    inset: f64,
    speed: f64,
) {
    assert_eq!(actual.filament_ironing_flow, flow);
    assert_eq!(actual.filament_ironing_spacing, OrcaFloat(spacing));
    assert_eq!(actual.filament_ironing_inset, OrcaFloat(inset));
    assert_eq!(actual.filament_ironing_speed, OrcaFloat(speed));
}
