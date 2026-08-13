use crate::{
    ProcessInfillPattern,
    geometry::{CoordinateScale, Point, Polygon, Polyline},
    project_slice::{
        prepare_infill::{bridge_over_infill, external_surfaces},
        tests::support::KsrArchive,
    },
};

use super::horizontal_shell_propagation;
use bridge_over_infill::candidate_bridge_angle::determine_candidate_bridge_angle;

#[test]
fn task22o60_real_ksr_uses_candidate_region_and_retained_rotation_without_mutation() {
    let prepared = prepare();
    let before = snapshot(&prepared);
    let region_before = first_region(&prepared).clone();
    let contexts = contexts(&prepared);
    assert_eq!(contexts.len(), 43);
    assert!(contexts.iter().all(|context| {
        context.pattern == ProcessInfillPattern::CrossHatch
            && context.rotation_bits == 0.0_f64.to_bits()
    }));

    let first = output_bits(&prepared);
    let second = output_bits(&prepared);

    assert_eq!(first, vec![0x3f50_624d_d2f1_a9fc; 43]);
    assert_eq!(second, first);
    assert_eq!(snapshot(&prepared), before);
    assert_eq!(first_region(&prepared), &region_before);
    bridge_over_infill::dispose(prepared);
}

fn prepare() -> bridge_over_infill::PreparedPostBridgeCandidates {
    let horizontal = horizontal_shell_propagation::fixture::prepare(KsrArchive::new().bytes());
    let external = external_surfaces::prepare(horizontal).unwrap();
    bridge_over_infill::prepare(external).unwrap()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Context {
    pattern: ProcessInfillPattern,
    rotation_bits: u64,
}

fn contexts(prepared: &bridge_over_infill::PreparedPostBridgeCandidates) -> Vec<Context> {
    let horizontal = &prepared.predecessor.predecessor;
    let traversal_object = &horizontal.predecessor.objects[0];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let (_, inputs) = prelude.object.as_parts();

    prepared.objects[0]
        .surfaces_by_layer
        .values()
        .flatten()
        .map(|candidate| {
            let input = inputs[candidate.source.layer_index].as_ref().unwrap();
            let region = prelude.object.region_options(input);
            Context {
                pattern: region.sparse_infill_pattern,
                rotation_bits: input.model_rotation_rad.to_bits(),
            }
        })
        .collect()
}

fn output_bits(prepared: &bridge_over_infill::PreparedPostBridgeCandidates) -> Vec<u64> {
    let horizontal = &prepared.predecessor.predecessor;
    let traversal_object = &horizontal.predecessor.objects[0];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let (_, inputs) = prelude.object.as_parts();
    let area = [Polygon::new(vec![Point::new(0, 0), Point::new(1, 0)])];
    let anchors = [Polyline::new(vec![Point::new(8, 9)])];

    prepared.objects[0]
        .surfaces_by_layer
        .values()
        .flatten()
        .map(|candidate| {
            let input = inputs[candidate.source.layer_index].as_ref().unwrap();
            determine_candidate_bridge_angle(
                &area,
                &anchors,
                &[],
                prelude.object.region_options(input),
                input.model_rotation_rad,
                CoordinateScale::Normal,
            )
            .to_bits()
        })
        .collect()
}

fn snapshot(prepared: &bridge_over_infill::PreparedPostBridgeCandidates) -> Vec<u64> {
    let mut bits = Vec::new();
    for (&layer_index, candidates) in &prepared.objects[0].surfaces_by_layer {
        bits.extend([layer_index as u64, candidates.len() as u64]);
        for candidate in candidates {
            bits.extend([
                candidate.source.layer_index as u64,
                candidate.source.region_index as u64,
                candidate.source.surface_index as u64,
                candidate.bridge_angle.to_bits(),
                candidate.new_polygons.len() as u64,
            ]);
            for polygon in &candidate.new_polygons {
                bits.push(polygon.points().len() as u64);
                bits.extend(
                    polygon
                        .points()
                        .iter()
                        .flat_map(|point| [point.x() as u64, point.y() as u64]),
                );
            }
        }
    }
    bits.extend(
        contexts(prepared)
            .into_iter()
            .map(|context| context.rotation_bits),
    );
    bits
}

fn first_region(
    prepared: &bridge_over_infill::PreparedPostBridgeCandidates,
) -> &crate::RegionOptions {
    let horizontal = &prepared.predecessor.predecessor;
    let traversal_object = &horizontal.predecessor.objects[0];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let (_, inputs) = prelude.object.as_parts();
    prelude
        .object
        .region_options(inputs.iter().flatten().next().unwrap())
}
