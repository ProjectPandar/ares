use crate::{
    ProcessInfillPattern,
    geometry::{Point, Polygon, Polyline},
    project_slice::{
        perimeters::flow::resolve_thick_solid_infill_bridge_flow,
        prepare_infill::{bridge_over_infill, external_surfaces},
        tests::support::KsrArchive,
    },
};

use super::horizontal_shell_propagation;
use bridge_over_infill::{
    candidate_anchored_bridge::construct_candidate_anchored_bridge,
    candidate_bridge_angle::determine_candidate_bridge_angle,
};

#[test]
fn task22o61_real_ksr_provenance_supplies_exact_flow_angle_scale_without_mutation() {
    let prepared = prepare();
    let horizontal = &prepared.predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    let traversal_object = &traversal.objects[0];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let (_, inputs) = prelude.object.as_parts();
    let candidate = prepared.objects[0]
        .surfaces_by_layer
        .values()
        .flatten()
        .next()
        .unwrap();
    let input = inputs[candidate.source.layer_index].as_ref().unwrap();
    let region = prelude.object.region_options(input);
    let nozzles = &traversal.resolved.views.full.project.print.nozzle_diameter;
    let flow = resolve_thick_solid_infill_bridge_flow(region, nozzles).unwrap();
    let region_before = region.clone();
    let nozzles_before = nozzles.clone();
    let scale_before = traversal.scale;
    assert_eq!(
        region.sparse_infill_pattern,
        ProcessInfillPattern::CrossHatch
    );
    assert_eq!(
        [
            flow.width.to_bits(),
            flow.height.to_bits(),
            flow.spacing.to_bits(),
            flow.nozzle_diameter.to_bits(),
        ],
        [0x3ecc_cccd, 0x3ecc_cccd, 0x3ee6_6667, 0x3ecc_cccd]
    );

    let no_sample = [polygon(&[(0, 0), (1, 0)])];
    let one_point = [polyline(&[(8, 9)])];
    let angle = determine_candidate_bridge_angle(
        &no_sample,
        &one_point,
        &[],
        region,
        input.model_rotation_rad,
        traversal.scale,
    );
    assert_eq!(angle.to_bits(), 0x3f50_624d_d2f1_a9fc);

    let area = [polygon(&[
        (0, 0),
        (2_000_000, 0),
        (2_000_000, 1_600_000),
        (0, 1_600_000),
    ])];
    let boundaries = vec![
        polyline(&[(-500_000, -300_000), (2_500_000, -300_000)]),
        polyline(&[(-500_000, 1_900_000), (2_500_000, 1_900_000)]),
    ];
    let run = || {
        construct_candidate_anchored_bridge(
            &area,
            boundaries.clone(),
            &[],
            &[],
            flow,
            angle,
            traversal.scale,
        )
        .unwrap()
    };
    let first = run();
    let second = run();

    assert_eq!(second, first);
    assert_eq!(first.boundary_polylines, boundaries);
    assert_eq!(first.bridging_area.len(), 1);
    assert_eq!(first.bridging_area[0].points().len(), 8);
    assert_eq!(region, &region_before);
    assert_eq!(nozzles, &nozzles_before);
    assert_eq!(traversal.scale, scale_before);
    bridge_over_infill::dispose(prepared);
}

fn prepare() -> bridge_over_infill::PreparedPostBridgeCandidates {
    let horizontal = horizontal_shell_propagation::fixture::prepare(KsrArchive::new().bytes());
    let external = external_surfaces::prepare(horizontal).unwrap();
    bridge_over_infill::prepare(external).unwrap()
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn polyline(points: &[(i64, i64)]) -> Polyline {
    Polyline::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}
