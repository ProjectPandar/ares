use crate::{
    ObjectOptions, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, Percent, ProcessBrimType,
    ProjectSettings, ProjectVolumeType, RegionOptions, SliceError,
    project::effective_config::resolve_bounded_project_config,
};

use super::{
    super::{
        extruders::collect_project_object_extruders,
        parameters::{SlicingParameters, slicing_parameters},
    },
    support::{object_options, project_with_range, region, resolved, source},
};

#[test]
fn task22a_object_extruders_cover_six_feature_gates_and_print_wide_brim() {
    let base = gated_region();
    for (wall_loops, expected) in [(0, vec![]), (1, vec![0]), (2, vec![0, 1])] {
        let mut candidate = base.clone();
        candidate.wall_loops = OrcaInt(wall_loops);
        assert_eq!(region_extruders(candidate), expected);
    }

    let mut sparse = base.clone();
    sparse.sparse_infill_density = Percent(20.0);
    assert_eq!(region_extruders(sparse), vec![2, 3]);
    let mut top = base.clone();
    top.top_shell_layers = OrcaInt(1);
    assert_eq!(region_extruders(top), vec![3, 4]);
    let mut bottom = base.clone();
    bottom.bottom_shell_layers = OrcaInt(1);
    assert_eq!(region_extruders(bottom), vec![3, 5]);

    for selector in [0, -7, 99] {
        let mut candidate = base.clone();
        candidate.wall_loops = OrcaInt(1);
        candidate.outer_wall_filament_id = OrcaInt(selector);
        assert_eq!(region_extruders(candidate), vec![0]);
    }

    let mut brim_region = base;
    brim_region.outer_wall_filament_id = OrcaInt(6);
    let print_wide = |brim_object| {
        collect_project_object_extruders(
            &[source(None, &[]), source(None, &[])],
            &[
                resolved(0, no_brim_object(), vec![brim_region.clone()]),
                resolved(1, brim_object, Vec::new()),
            ],
            6,
        )
    };
    let mut qualifying = no_brim_object();
    qualifying.brim_type = ProcessBrimType::AutoBrim;
    assert_eq!(print_wide(qualifying), vec![vec![5], Vec::new()]);
    let mut explicit = no_brim_object();
    explicit.brim_type = ProcessBrimType::OuterOnly;
    explicit.brim_width = OrcaFloat(2.0);
    assert_eq!(print_wide(explicit), vec![vec![5], Vec::new()]);
    let mut zero_width = no_brim_object();
    zero_width.brim_type = ProcessBrimType::OuterOnly;
    assert_eq!(
        print_wide(zero_width),
        vec![Vec::<usize>::new(), Vec::new()]
    );
    let mut disabled = no_brim_object();
    disabled.brim_width = OrcaFloat(2.0);
    assert_eq!(print_wide(disabled), vec![Vec::<usize>::new(), Vec::new()]);
}

#[test]
fn task22a_object_extruders_include_model_modifier_and_object_fallbacks() {
    let sources = [
        source(
            Some(4),
            &[
                (ProjectVolumeType::ModelPart, Some(2)),
                (ProjectVolumeType::ParameterModifier, Some(3)),
                (ProjectVolumeType::ModelPart, Some(0)),
                (ProjectVolumeType::NegativeVolume, Some(5)),
                (ProjectVolumeType::SupportEnforcer, Some(5)),
                (ProjectVolumeType::SupportBlocker, Some(5)),
                (ProjectVolumeType::ModelPart, Some(6)),
            ],
        ),
        source(None, &[(ProjectVolumeType::ModelPart, None)]),
        source(Some(0), &[(ProjectVolumeType::ParameterModifier, None)]),
    ];
    let resolved = [
        resolved(0, object_options(), Vec::new()),
        resolved(1, object_options(), Vec::new()),
        resolved(2, object_options(), Vec::new()),
    ];

    assert_eq!(
        collect_project_object_extruders(&sources, &resolved, 5),
        vec![vec![1, 2, 3, 5], vec![0], Vec::new()]
    );
}

#[test]
fn task22a_range_extruder_reaches_nozzles_only_through_occupied_feature_fallback() {
    let occupied = project_with_range(0.0, 1.0, 2);
    let nonintersecting = project_with_range(200.0, 201.0, 2);

    let occupied_resolved = resolve_bounded_project_config(&occupied).unwrap();
    let nonintersecting_resolved = resolve_bounded_project_config(&nonintersecting).unwrap();
    assert_eq!(occupied_resolved.usage.supported_used_filaments, vec![0, 1]);
    assert_eq!(
        nonintersecting_resolved.usage.supported_used_filaments,
        vec![0, 1]
    );
    assert_eq!(
        collect_project_object_extruders(
            occupied.objects(),
            &occupied_resolved.objects,
            occupied_resolved.logical_filament_count,
        ),
        vec![vec![0, 1]]
    );
    assert_eq!(
        collect_project_object_extruders(
            nonintersecting.objects(),
            &nonintersecting_resolved.objects,
            nonintersecting_resolved.logical_filament_count,
        ),
        vec![vec![0]]
    );
}

#[test]
fn task22a_nozzle_lookup_preserves_zero_one_two_and_out_of_range_fallback() {
    let settings = settings(&[0.4, 0.8], &[0.04, 0.11], &[0.21, 0.55], 0.2);
    let object = object_with_height(0.2);
    let first = expected(0.2, 0.2, 10.0, 0.04, 0.21);
    let second = expected(0.2, 0.2, 10.0, 0.11, 0.55);

    assert_eq!(
        slicing_parameters(&settings, &object, 10.0, &[0]),
        Ok(first.clone())
    );
    assert_eq!(
        slicing_parameters(&settings, &object, 10.0, &[1]),
        Ok(first.clone())
    );
    assert_eq!(
        slicing_parameters(&settings, &object, 10.0, &[2]),
        Ok(second)
    );
    assert_eq!(
        slicing_parameters(&settings, &object, 10.0, &[3]),
        Ok(first)
    );
}

#[test]
fn task22a_nozzle_limits_apply_defaults_clamps_and_multi_source_aggregation() {
    let defaults = settings(&[0.4, 0.8], &[0.0, 0.005], &[0.0, 0.0], 0.2);
    assert_eq!(
        slicing_parameters(&defaults, &object_with_height(0.1), 8.0, &[1]),
        Ok(expected(0.1, 0.2, 8.0, 0.07, 0.75 * 0.4))
    );
    assert_eq!(
        slicing_parameters(&defaults, &object_with_height(0.05), 8.0, &[1]),
        Ok(expected(0.05, 0.2, 8.0, 0.05, 0.75 * 0.4))
    );
    assert_eq!(
        slicing_parameters(&defaults, &object_with_height(0.1), 8.0, &[2]),
        Ok(expected(0.1, 0.2, 8.0, 0.01, 0.75 * 0.8))
    );

    let max_below_min = settings(&[0.4, 0.8], &[0.0, 0.005], &[0.0, 0.005], 0.2);
    assert_eq!(
        slicing_parameters(&max_below_min, &object_with_height(0.005), 8.0, &[2]),
        Ok(expected(0.005, 0.2, 8.0, 0.005, 0.01))
    );

    let aggregation = settings(&[0.4, 0.8], &[0.12, 0.04], &[0.3, 0.15], 0.2);
    assert_eq!(
        slicing_parameters(&aggregation, &object_with_height(0.2), 8.0, &[1, 2]),
        Ok(expected(0.2, 0.2, 8.0, 0.12, 0.2))
    );
    let empty_source = settings(&[0.4, 0.8], &[0.04, 0.12], &[0.3, 0.25], 0.2);
    assert_eq!(
        slicing_parameters(&empty_source, &object_with_height(0.2), 8.0, &[]),
        Ok(expected(0.2, 0.2, 8.0, 0.04, 0.3))
    );
}

#[test]
fn task22a_filament_map_does_not_affect_nozzle_limits() {
    let mut identity = settings(&[0.4, 0.8], &[0.04, 0.12], &[0.3, 0.15], 0.2);
    identity.project.gcode.filament_map = OrcaInts(vec![OrcaInt(1), OrcaInt(2)]);
    let mut remapped = identity.clone();
    remapped.project.gcode.filament_map = OrcaInts(vec![OrcaInt(2), OrcaInt(1)]);
    let object = object_with_height(0.2);

    let identity = slicing_parameters(&identity, &object, 8.0, &[1]).unwrap();
    let remapped = slicing_parameters(&remapped, &object, 8.0, &[1]).unwrap();
    assert_eq!(identity, expected(0.2, 0.2, 8.0, 0.04, 0.3));
    assert_eq!(remapped, identity);
}

#[test]
fn task22a_first_layer_height_uses_positive_value_or_regular_fallback() {
    let object = object_with_height(0.18);
    for (initial, expected_first) in [(0.3, 0.3), (0.0, 0.18), (-0.2, 0.18), (f64::NAN, 0.18)] {
        let settings = settings(&[0.4], &[0.07], &[0.28], initial);
        assert_eq!(
            slicing_parameters(&settings, &object, 4.0, &[0]),
            Ok(expected(0.18, expected_first, 4.0, 0.07, 0.28))
        );
    }
}

#[test]
fn task22a_invalid_slicing_parameter_numbers_are_keyed() {
    let valid = settings(&[0.4], &[0.07], &[0.28], 0.2);
    for layer_height in [0.0, -0.2, f64::INFINITY, f64::NAN] {
        assert_invalid_key(
            slicing_parameters(&valid, &object_with_height(layer_height), 4.0, &[0]),
            "layer_height",
        );
    }

    for initial in [f64::INFINITY, f64::NEG_INFINITY] {
        let settings = settings(&[0.4], &[0.07], &[0.28], initial);
        let key = if initial.is_sign_positive() {
            "initial_layer_print_height"
        } else {
            ""
        };
        if key.is_empty() {
            assert_eq!(
                slicing_parameters(&settings, &object_with_height(0.2), 4.0, &[0]),
                Ok(expected(0.2, 0.2, 4.0, 0.07, 0.28))
            );
        } else {
            assert_invalid_key(
                slicing_parameters(&settings, &object_with_height(0.2), 4.0, &[0]),
                key,
            );
        }
    }

    for object_height in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert_invalid_key(
            slicing_parameters(&valid, &object_with_height(0.2), object_height, &[0]),
            "project-object Z bounds",
        );
    }
}

fn settings(nozzles: &[f64], minimums: &[f64], maximums: &[f64], initial: f64) -> ProjectSettings {
    let mut settings = ProjectSettings::default();
    settings.project.print.nozzle_diameter = floats(nozzles);
    settings.project.print.min_layer_height = floats(minimums);
    settings.project.print.max_layer_height = floats(maximums);
    settings.process.print.initial_layer_print_height = OrcaFloat(initial);
    settings
}

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}

fn object_with_height(layer_height: f64) -> ObjectOptions {
    let mut object = object_options();
    object.layer_height = OrcaFloat(layer_height);
    object
}

fn gated_region() -> RegionOptions {
    let mut candidate = region();
    candidate.wall_loops = OrcaInt(0);
    candidate.sparse_infill_density = Percent(0.0);
    candidate.top_shell_layers = OrcaInt(0);
    candidate.bottom_shell_layers = OrcaInt(0);
    candidate.outer_wall_filament_id = OrcaInt(1);
    candidate.inner_wall_filament_id = OrcaInt(2);
    candidate.sparse_infill_filament_id = OrcaInt(3);
    candidate.internal_solid_filament_id = OrcaInt(4);
    candidate.top_surface_filament_id = OrcaInt(5);
    candidate.bottom_surface_filament_id = OrcaInt(6);
    candidate
}

fn no_brim_object() -> ObjectOptions {
    let mut object = object_options();
    object.brim_type = ProcessBrimType::NoBrim;
    object.brim_width = OrcaFloat(0.0);
    object
}

fn region_extruders(candidate: RegionOptions) -> Vec<usize> {
    collect_project_object_extruders(
        &[source(None, &[])],
        &[resolved(0, no_brim_object(), vec![candidate])],
        6,
    )
    .pop()
    .unwrap()
}

fn expected(
    layer_height: f64,
    first_height: f64,
    object_height: f64,
    min_layer_height: f64,
    max_layer_height: f64,
) -> SlicingParameters {
    SlicingParameters {
        base_raft_layers: 0,
        interface_raft_layers: 0,
        base_raft_layer_height: 0.0,
        interface_raft_layer_height: 0.0,
        contact_raft_layer_height: 0.0,
        layer_height,
        min_layer_height,
        max_layer_height,
        first_print_layer_height: first_height,
        first_object_layer_height: first_height,
        first_object_layer_bridging: false,
        gap_raft_object: 0.0,
        gap_object_support: 0.0,
        gap_support_object: 0.0,
        raft_base_top_z: 0.0,
        raft_interface_top_z: 0.0,
        raft_contact_top_z: 0.0,
        object_print_z_min: 0.0,
        object_print_z_max: object_height,
        object_print_z_uncompensated_max: object_height,
        object_shrinkage_compensation_z: 1.0,
    }
}

fn assert_invalid_key(result: Result<SlicingParameters, SliceError>, key: &str) {
    let SliceError::InvalidInput(message) = result.unwrap_err() else {
        panic!("expected keyed InvalidInput");
    };
    assert!(
        message.contains(key),
        "{message:?} does not contain {key:?}"
    );
}
