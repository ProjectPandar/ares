use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloat, OrcaFloats, OrcaInt, Percent, ProjectSettings,
    RegionOptions, SliceError,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project::effective_config::types::ResolvedProjectObject,
    project_slice::{
        compensation::{
            PostCompensationPrintObject, ValidatedTask22mConfig, apply_project_compensation,
            validate_task22m_configs,
        },
        layers::{PlannedLayer, PlannedPrintObject},
        region_slices::{PostRegion, PostRegionPrintObject, RegionLayer, RegionSurface},
    },
    slice_project,
};

use super::super::support::{KsrArchive, identity_resolved, metadata, region};

const GEOMETRY_ERROR: &str = "project elephant-foot compensation geometry is nonfinite or outside the supported Clipper range";

#[test]
fn task22m_flow_validates_raw_config_before_unsigned_conversion() {
    let mut options = object_options();
    options.elefant_foot_compensation = OrcaFloat(0.15);
    options.elefant_foot_compensation_layers = OrcaInt(3);
    options.raft_layers = OrcaInt(-1);
    options.line_width = FloatOrPercent::Percent(Percent(110.0));

    assert_eq!(
        validate_task22m_configs(&[&options]).unwrap(),
        [ValidatedTask22mConfig {
            compensation_mm: 0.15,
            compensation_layers: 3,
            raft_layers: -1,
            object_line_width: FloatOrPercent::Percent(Percent(110.0)),
        }]
    );

    options.elefant_foot_compensation = OrcaFloat(0.0);
    options.elefant_foot_compensation_layers = OrcaInt(1);
    assert_eq!(
        validate_task22m_configs(&[&options]).unwrap()[0].compensation_mm,
        0.0
    );

    for compensation in [-1.0, f64::NAN, f64::INFINITY] {
        options.elefant_foot_compensation = OrcaFloat(compensation);
        options.elefant_foot_compensation_layers = OrcaInt(0);
        assert_invalid(
            validate_task22m_configs(&[&options]),
            "invalid Orca option elefant_foot_compensation",
        );
    }

    options.elefant_foot_compensation = OrcaFloat(0.15);
    for layers in [0, -1, i32::MIN] {
        options.elefant_foot_compensation_layers = OrcaInt(layers);
        assert_invalid(
            validate_task22m_configs(&[&options]),
            "invalid Orca option elefant_foot_compensation_layers",
        );
    }
}

#[test]
fn task22m_flow_orders_xy_feature_keys() {
    let mut options = object_options();
    options.xy_hole_compensation = OrcaFloat(0.1);
    options.xy_contour_compensation = OrcaFloat(0.2);
    assert_unsupported(
        validate_task22m_configs(&[&options]),
        "xy_hole_compensation",
    );

    options.xy_hole_compensation = OrcaFloat(0.0);
    assert_unsupported(
        validate_task22m_configs(&[&options]),
        "xy_contour_compensation",
    );
}

#[test]
fn task22m_flow_preflight_is_all_or_error() {
    let mut first = object_options();
    first.elefant_foot_compensation = OrcaFloat(0.1);
    first.raft_layers = OrcaInt(1);
    first.line_width = FloatOrPercent::Float(0.4);
    let first_before = first.clone();

    let mut second = object_options();
    second.elefant_foot_compensation = OrcaFloat(0.2);
    second.elefant_foot_compensation_layers = OrcaInt(0);
    second.raft_layers = OrcaInt(-1);
    second.line_width = FloatOrPercent::Percent(Percent(105.0));
    let second_before = second.clone();

    assert_invalid(
        validate_task22m_configs(&[&first, &second]),
        "invalid Orca option elefant_foot_compensation_layers",
    );
    assert_eq!(first, first_before);
    assert_eq!(second, second_before);

    second.elefant_foot_compensation_layers = OrcaInt(2);
    assert_eq!(
        validate_task22m_configs(&[&first, &second]).unwrap(),
        [
            ValidatedTask22mConfig {
                compensation_mm: 0.1,
                compensation_layers: 1,
                raft_layers: 1,
                object_line_width: FloatOrPercent::Float(0.4),
            },
            ValidatedTask22mConfig {
                compensation_mm: 0.2,
                compensation_layers: 2,
                raft_layers: -1,
                object_line_width: FloatOrPercent::Percent(Percent(105.0)),
            },
        ]
    );
}

#[tokio::test]
async fn task22m_flow_public_raft_gate_precedes_task22m_for_both_signs() {
    for raft_layers in ["1", "-1"] {
        let mut archive = KsrArchive::new();
        archive.replace(
            "Metadata/project_settings.config",
            "\t\"raft_layers\": \"0\",",
            &format!("\t\"raft_layers\": \"{raft_layers}\","),
        );

        assert_eq!(
            slice_project(archive.bytes(), metadata())
                .await
                .unwrap_err(),
            SliceError::UnsupportedProjectFeature("raft_layers".to_owned())
        );
    }
}

#[test]
fn task22m_transaction_preflight_phase_order_is_global() {
    let first = task22m_options(0.15, 1, 0);
    let second = task22m_options(0.15, 0, 0);
    let objects = vec![post_object(0, 1, 2, None), post_object(1, 1, 1, None)];

    assert_apply_error(
        apply(
            objects,
            vec![resolved(0, first), resolved(1, second)],
            &[0.4],
        ),
        SliceError::InvalidInput("invalid Orca option elefant_foot_compensation_layers".to_owned()),
    );
    let mut invalid_flow_options = task22m_options(0.15, 1, 0);
    invalid_flow_options.line_width = FloatOrPercent::Percent(Percent(0.0));
    let objects = vec![invalid_flow_object(0), post_object(1, 1, 2, None)];

    assert_apply_error(
        apply(
            objects,
            vec![
                resolved(0, invalid_flow_options),
                resolved(1, task22m_options(0.15, 1, 0)),
            ],
            &[0.4],
        ),
        SliceError::UnsupportedProjectFeature("multi_region_layer_slices".to_owned()),
    );
    let mut invalid_flow_options = task22m_options(0.15, 1, 0);
    invalid_flow_options.line_width = FloatOrPercent::Percent(Percent(0.0));
    let objects = vec![
        post_object(0, 1, 1, Some(out_of_range_polygon())),
        invalid_flow_object(1),
    ];

    assert_apply_error(
        apply(
            objects,
            vec![
                resolved(0, task22m_options(0.15, 1, 0)),
                resolved(1, invalid_flow_options),
            ],
            &[0.4],
        ),
        SliceError::InvalidInput("invalid external perimeter flow spacing".to_owned()),
    );
}

#[test]
fn task22m_transaction_allows_zero_layer_multi_region_objects() {
    let output = apply(
        vec![post_object(0, 0, 2, None)],
        vec![resolved(0, task22m_options(0.15, 1, 0))],
        &[],
    )
    .unwrap();
    let (_, lslices) = output.into_iter().next().unwrap().into_parts();

    assert!(lslices.is_empty());
}

#[test]
fn task22m_transaction_region_cardinality_controls_flow_requirement() {
    let output = apply(
        vec![post_object(0, 2, 0, None)],
        vec![resolved(0, task22m_options(0.15, 2, 0))],
        &[],
    )
    .unwrap();
    let (_, lslices) = output.into_iter().next().unwrap().into_parts();

    assert_eq!(lslices, [Vec::new(), Vec::new()]);
    assert_apply_error(
        apply(
            vec![post_object(0, 1, 1, None)],
            vec![resolved(0, task22m_options(0.15, 1, 0))],
            &[],
        ),
        SliceError::InvalidInput("invalid Orca option nozzle_diameter".to_owned()),
    );
}

#[test]
fn task22m_transaction_scaled_nonfinite_precedes_flow_but_raft_skips_both() {
    assert_apply_error(
        apply(
            vec![post_object(0, 1, 1, None)],
            vec![resolved(0, task22m_options(f64::MAX, 1, 0))],
            &[],
        ),
        SliceError::InvalidInput(GEOMETRY_ERROR.to_owned()),
    );

    for raft_layers in [-1, 1] {
        let raw = square();
        let output = apply(
            vec![post_object(0, 1, 1, Some(raw.clone()))],
            vec![resolved(0, task22m_options(f64::MAX, 1, raft_layers))],
            &[],
        )
        .unwrap();
        let (_, lslices) = output.into_iter().next().unwrap().into_parts();
        assert_eq!(lslices, [vec![raw]]);
    }
}

#[test]
fn task22m_transaction_later_geometry_failure_exposes_no_partial_output() {
    let objects = vec![
        post_object(0, 1, 1, Some(square())),
        post_object(1, 1, 1, Some(out_of_range_polygon())),
    ];

    assert_apply_error(
        apply(
            objects,
            vec![
                resolved(0, task22m_options(0.15, 1, 0)),
                resolved(1, task22m_options(0.15, 1, 0)),
            ],
            &[0.4],
        ),
        SliceError::InvalidInput(GEOMETRY_ERROR.to_owned()),
    );
}

fn assert_invalid(result: Result<Vec<ValidatedTask22mConfig>, SliceError>, expected: &str) {
    assert_eq!(
        result.unwrap_err(),
        SliceError::InvalidInput(expected.to_owned())
    );
}
fn assert_unsupported(result: Result<Vec<ValidatedTask22mConfig>, SliceError>, expected: &str) {
    assert_eq!(
        result.unwrap_err(),
        SliceError::UnsupportedProjectFeature(expected.to_owned())
    );
}
fn object_options() -> ObjectOptions {
    ObjectOptions::from_base(&ProjectSettings::default().process.object)
}
fn apply(
    objects: Vec<PostRegionPrintObject>,
    resolved_objects: Vec<ResolvedProjectObject>,
    nozzles: &[f64],
) -> Result<Vec<PostCompensationPrintObject>, SliceError> {
    apply_project_compensation(
        objects,
        &resolved_objects,
        FloatOrPercent::Float(0.0),
        &OrcaFloats(nozzles.iter().copied().map(OrcaFloat).collect()),
        CoordinateScale::Normal,
    )
}
fn assert_apply_error(
    result: Result<Vec<PostCompensationPrintObject>, SliceError>,
    expected: SliceError,
) {
    match result {
        Err(error) => assert_eq!(error, expected),
        Ok(_) => panic!("Task 22M apply unexpectedly succeeded"),
    }
}
fn resolved(source_object_index: usize, options: ObjectOptions) -> ResolvedProjectObject {
    let mut resolved = identity_resolved(source_object_index);
    resolved.object = options;
    resolved
}
fn task22m_options(compensation: f64, layers: i32, raft_layers: i32) -> ObjectOptions {
    let mut options = object_options();
    options.elefant_foot_compensation = OrcaFloat(compensation);
    options.elefant_foot_compensation_layers = OrcaInt(layers);
    options.raft_layers = OrcaInt(raft_layers);
    options.line_width = FloatOrPercent::Float(0.4);
    options
}
fn flow_region() -> RegionOptions {
    let mut options = region();
    options.outer_wall_line_width = FloatOrPercent::Float(0.5);
    options.outer_wall_filament_id = OrcaInt(1);
    options
}

fn invalid_flow_object(source_object_index: usize) -> PostRegionPrintObject {
    let mut object = post_object(source_object_index, 1, 1, None);
    object.regions[0].options.outer_wall_line_width = FloatOrPercent::Percent(Percent(0.0));
    object
}

fn post_object(
    source_object_index: usize,
    layer_count: usize,
    region_count: usize,
    first_surface: Option<ExPolygon>,
) -> PostRegionPrintObject {
    PostRegionPrintObject {
        plan: PlannedPrintObject {
            source_object_index,
            transform_index: 0,
            layers: (0..layer_count)
                .map(|id| PlannedLayer {
                    id,
                    height: 0.2,
                    print_z: (id + 1) as f64 * 0.2,
                    slice_z: id as f64 * 0.2 + 0.1,
                })
                .collect(),
        },
        volume_slices: Vec::new(),
        regions: (0..region_count)
            .map(|id| PostRegion {
                id,
                options: flow_region(),
                layers: (0..layer_count)
                    .map(|layer_index| RegionLayer {
                        surfaces: (id == 0 && layer_index == 0)
                            .then(|| first_surface.clone())
                            .flatten()
                            .into_iter()
                            .map(RegionSurface::internal)
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn square() -> ExPolygon {
    expolygon(&[
        (0, 0),
        (1_000_000, 0),
        (1_000_000, 1_000_000),
        (0, 1_000_000),
    ])
}

fn out_of_range_polygon() -> ExPolygon {
    expolygon(&[(i64::MIN, 0), (0, 0), (0, 1), (i64::MIN, 1)])
}

fn expolygon(points: &[(i64, i64)]) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect()),
        Vec::new(),
    )
}
