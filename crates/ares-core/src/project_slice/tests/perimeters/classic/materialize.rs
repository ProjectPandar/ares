mod deep;
mod series;

use crate::{
    SliceError,
    project_slice::perimeters::{
        classic::{
            materialize::ExtrusionRole,
            traversal::{LowerFlowRoute, PendingPathBranch, TraversalSeed},
        },
        prepare_post_classic_raw_paths, prepare_post_classic_traversal,
    },
    slice_project,
};

use super::super::super::support::{ksr_project, metadata};

#[test]
fn task22o7_ksr_materializes_real_branches_with_exact_numeric_provenance() {
    let prepared = prepare_post_classic_raw_paths(ksr_project()).unwrap();
    let scale = prepared.predecessor.scale;
    let mut roles = [false; 3];
    let mut routes = [false; 3];
    let mut checked_final_series = [false; 3];
    let mut saw_ordinary = false;
    let mut saw_overhang = false;
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    for (raw_object, object) in prepared.objects.iter().zip(&prepared.predecessor.objects) {
        assert_eq!(raw_object.records.len(), object.records.len());
        for (record_index, (raw_record, record)) in
            raw_object.records.iter().zip(&object.records).enumerate()
        {
            assert_eq!(raw_record.is_some(), record.is_some());
            let (Some(raw_record), Some(record)) = (raw_record, record) else {
                continue;
            };
            saw_ordinary |= matches!(record.branch, PendingPathBranch::OrdinaryUnsplit { .. });
            saw_overhang |= matches!(record.branch, PendingPathBranch::OverhangClipping { .. });
            assert_eq!(raw_record.surfaces.len(), record.surfaces.len());
            for (raw_surface, surface) in raw_record.surfaces.iter().zip(&record.surfaces) {
                assert_eq!(raw_surface.source_index, surface.source_index);
                assert_nodes(
                    &raw_surface.roots,
                    &surface.roots,
                    &mut AssertContext {
                        record,
                        object,
                        record_index,
                        scale,
                        roles: &mut roles,
                        routes: &mut routes,
                        checked_final_series: &mut checked_final_series,
                    },
                );
            }
        }
    }
    assert!(saw_ordinary && saw_overhang);
    assert!(roles[0] && roles[1] && roles[2]);
    assert!(routes.into_iter().filter(|seen| *seen).count() >= 2);
    assert!(
        checked_final_series
            .into_iter()
            .filter(|seen| *seen)
            .count()
            >= 2
    );
}

#[test]
fn task22o7_ksr_xyz_and_path_order_checksum_is_deterministic() {
    assert_eq!(checksum(ksr_project()), checksum(ksr_project()));
    assert_ne!(checksum(ksr_project()), 0);
}

#[test]
fn task22o7_coordinate_error_is_transactional_and_has_no_ordinary_fallback() {
    let mut prepared = prepare_post_classic_traversal(ksr_project()).unwrap();
    let record = prepared
        .objects
        .iter_mut()
        .flat_map(|object| object.records.iter_mut())
        .flatten()
        .find(|record| {
            matches!(record.branch, PendingPathBranch::OverhangClipping { .. })
                && record
                    .surfaces
                    .iter()
                    .any(|surface| !surface.roots.is_empty())
        })
        .unwrap();
    let seed = record
        .surfaces
        .iter_mut()
        .find_map(|surface| surface.roots.first_mut())
        .unwrap();
    let high = 0x4000_0000_0000_0000_i64;
    seed.polygon = crate::geometry::Polygon::new(vec![
        crate::geometry::Point::new(high, 0),
        crate::geometry::Point::new(high + 10, 0),
        crate::geometry::Point::new(high + 10, 10),
    ]);
    assert!(matches!(
        crate::project_slice::perimeters::classic::materialize::finish(prepared),
        Err(SliceError::InvalidInput(message))
            if message == "classic perimeter raw path coordinate is outside the supported Clipper range"
    ));
}

#[tokio::test]
async fn task22o7_public_lifecycle_executes_raw_paths_then_stays_incomplete() {
    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

struct AssertContext<'a> {
    record: &'a crate::project_slice::perimeters::classic::traversal::ClassicTraversalRecord,
    object: &'a crate::project_slice::perimeters::classic::PostClassicTraversalPrintObject,
    record_index: usize,
    scale: crate::geometry::CoordinateScale,
    roles: &'a mut [bool; 3],
    routes: &'a mut [bool; 3],
    checked_final_series: &'a mut [bool; 3],
}

fn assert_nodes(
    raw_roots: &[crate::project_slice::perimeters::classic::materialize::RawPathNode],
    seeds: &[TraversalSeed],
    context: &mut AssertContext<'_>,
) {
    assert_eq!(raw_roots.len(), seeds.len());
    let mut pending = raw_roots.iter().zip(seeds).collect::<Vec<_>>();
    while let Some((raw, seed)) = pending.pop() {
        let route_index = match seed.route {
            LowerFlowRoute::SmallerExternal => 0,
            LowerFlowRoute::External => 1,
            LowerFlowRoute::Internal => 2,
        };
        context.routes[route_index] = true;
        if matches!(
            context.record.branch,
            PendingPathBranch::OverhangClipping { .. }
        ) && !context.checked_final_series[route_index]
        {
            let lower = context
                .object
                .lower_series(context.record_index, seed.route)
                .last()
                .expect("an overhang branch has a final lower-series element");
            let expected = crate::project_slice::perimeters::classic::materialize::path::materialize_overhang_from_lower(
                context.record,
                seed,
                context.scale,
                lower,
            )
            .unwrap();
            assert_eq!(raw.paths, expected);
            context.checked_final_series[route_index] = true;
        }
        for path in &raw.paths {
            assert_path_provenance(path, seed, context.record, context.roles);
        }
        assert_eq!(raw.children.len(), seed.children.len());
        pending.extend(raw.children.iter().zip(&seed.children));
    }
}

fn assert_path_provenance(
    path: &crate::project_slice::perimeters::classic::materialize::ExtrusionPath,
    seed: &TraversalSeed,
    record: &crate::project_slice::perimeters::classic::traversal::ClassicTraversalRecord,
    roles: &mut [bool; 3],
) {
    roles[match path.role {
        ExtrusionRole::ExternalPerimeter => 0,
        ExtrusionRole::Perimeter => 1,
        ExtrusionRole::OverhangPerimeter => 2,
        ExtrusionRole::GapFill => unreachable!("raw perimeter paths cannot be gap fill"),
    }] = true;
    assert!(path.polyline.points.iter().all(|point| point.z == 0));
    if path.role == ExtrusionRole::OverhangPerimeter {
        assert_eq!(
            path.mm3_per_mm.to_bits(),
            record.overhang_flow.mm3_per_mm.to_bits()
        );
        assert_eq!(path.width.to_bits(), record.overhang_flow.width.to_bits());
        assert_eq!(path.height.to_bits(), record.overhang_flow.height.to_bits());
    } else {
        assert_eq!(path.mm3_per_mm.to_bits(), seed.mm3_per_mm.to_bits());
        assert_eq!(path.width.to_bits(), seed.width.to_bits());
        assert_eq!(
            path.height.to_bits(),
            (record.layer_height as f32).to_bits()
        );
    }
}

fn checksum(input: impl AsRef<[u8]>) -> i128 {
    let prepared = prepare_post_classic_raw_paths(input).unwrap();
    let mut checksum = 0_i128;
    for object in &prepared.objects {
        for record in object.records.iter().flatten() {
            for surface in &record.surfaces {
                accumulate_surface_checksum(surface, &mut checksum);
            }
        }
    }
    checksum
}

fn accumulate_surface_checksum(
    surface: &crate::project_slice::perimeters::classic::materialize::PreparedRawPathSurface,
    checksum: &mut i128,
) {
    let mut pending = surface.roots.iter().rev().collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        accumulate_node_checksum(node, checksum);
        pending.extend(node.children.iter().rev());
    }
}

fn accumulate_node_checksum(
    node: &crate::project_slice::perimeters::classic::materialize::RawPathNode,
    checksum: &mut i128,
) {
    for path in &node.paths {
        *checksum = checksum
            .wrapping_mul(37)
            .wrapping_add(i128::from(path.width.to_bits()));
        for point in &path.polyline.points {
            *checksum = checksum.wrapping_mul(31).wrapping_add(
                i128::from(point.x) + 3 * i128::from(point.y) + 7 * i128::from(point.z),
            );
        }
    }
}
