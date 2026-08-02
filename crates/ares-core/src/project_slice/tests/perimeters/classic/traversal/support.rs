use crate::project_slice::perimeters::{
    classic::traversal::{LowerFlowRoute, PendingPathBranch, TraversalSeed},
    prepare_post_classic_traversal,
};

pub(super) use super::super::super::super::support::{ksr_project as project, metadata};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Summary {
    pub(super) source_index: usize,
    pub(super) roots: usize,
    pub(super) seeds: usize,
    pub(super) checksum: i128,
    pub(super) branch: PendingPathBranch,
    pub(super) diagnostics: (usize, usize),
}

pub(super) fn assert_record_alignment(input: impl AsRef<[u8]>) {
    let prepared = prepare_post_classic_traversal(input).unwrap();
    let mut saw_non_roundtripping_height = false;
    let mut saw_odd_layer = false;
    let mut saw_even_layer = false;
    for object in &prepared.objects {
        assert_eq!(object.records.len(), object.predecessor.records.len());
        let input_object = &object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        for ((traversal, hierarchy), input) in object
            .records
            .iter()
            .zip(&object.predecessor.records)
            .zip(input_object.as_parts().1)
        {
            assert_eq!(traversal.is_some(), hierarchy.is_some());
            assert_eq!(traversal.is_some(), input.is_some());
            if let (Some(traversal), Some(hierarchy), Some(input)) = (traversal, hierarchy, input) {
                assert_eq!(traversal.surfaces.len(), hierarchy.surfaces.len());
                assert_eq!(
                    traversal.layer_height.to_bits(),
                    input.layer_height.to_bits()
                );
                saw_non_roundtripping_height |=
                    input.layer_height != f64::from(input.layer_height as f32);

                let configured = input_object.region_options(input).overhang_reverse.0;
                let odd_layer = input.layer_id % 2 == 1;
                assert_eq!(traversal.overhang_reverse.configured, configured);
                assert_eq!(traversal.overhang_reverse.odd_layer, odd_layer);
                assert_eq!(traversal.overhang_reverse.active, configured && odd_layer);
                saw_odd_layer |= odd_layer;
                saw_even_layer |= !odd_layer;
            }
        }
    }
    assert!(saw_non_roundtripping_height);
    assert!(saw_odd_layer);
    assert!(saw_even_layer);
}

pub(super) fn summaries(input: impl AsRef<[u8]>) -> Vec<Summary> {
    let prepared = prepare_post_classic_traversal(input).unwrap();
    prepared
        .objects
        .iter()
        .flat_map(|object| {
            object
                .records
                .iter()
                .zip(&object.predecessor.records)
                .enumerate()
                .filter_map(move |(record_index, (traversal, hierarchy))| {
                    Some((
                        object,
                        record_index,
                        traversal.as_ref()?,
                        hierarchy.as_ref()?,
                    ))
                })
        })
        .flat_map(|(object, record_index, traversal, hierarchy)| {
            traversal.surfaces.iter().zip(&hierarchy.surfaces).map(
                move |(traversal_surface, hierarchy_surface)| {
                    assert_eq!(
                        traversal_surface.source_index,
                        hierarchy_surface.source_index
                    );
                    assert_seed_alignment(&traversal_surface.roots, &hierarchy_surface.roots);
                    assert_routes_resolve_to_predecessor(
                        object,
                        record_index,
                        traversal_surface.roots.as_slice(),
                    );
                    let (seeds, checksum) = seed_summary(&traversal_surface.roots);
                    Summary {
                        source_index: traversal_surface.source_index,
                        roots: traversal_surface.roots.len(),
                        seeds,
                        checksum,
                        branch: traversal.branch,
                        diagnostics: (
                            hierarchy_surface
                                .remaining_contours
                                .iter()
                                .map(Vec::len)
                                .sum(),
                            hierarchy_surface.remaining_holes.iter().map(Vec::len).sum(),
                        ),
                    }
                },
            )
        })
        .collect()
}

fn assert_seed_alignment(
    seeds: &[TraversalSeed],
    loops: &[crate::project_slice::perimeters::classic::hierarchy::PerimeterGeneratorLoop],
) {
    assert_eq!(seeds.len(), loops.len());
    let mut pending = seeds.iter().zip(loops).collect::<Vec<_>>();
    while let Some((seed, loop_)) = pending.pop() {
        assert_eq!(seed.polygon, loop_.polygon);
        assert_eq!(seed.depth, loop_.depth);
        assert_eq!(seed.is_contour, loop_.is_contour);
        assert_eq!(
            seed.is_smaller_width_perimeter,
            loop_.is_smaller_width_perimeter
        );
        assert_eq!(seed.children.len(), loop_.children.len());
        pending.extend(seed.children.iter().zip(&loop_.children));
    }
}

fn assert_routes_resolve_to_predecessor(
    object: &crate::project_slice::perimeters::classic::PostClassicTraversalPrintObject,
    record_index: usize,
    roots: &[TraversalSeed],
) {
    let prelude = object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records[record_index]
        .as_ref()
        .unwrap();
    let mut pending = roots.iter().collect::<Vec<_>>();
    while let Some(seed) = pending.pop() {
        let expected = match seed.route {
            LowerFlowRoute::SmallerExternal => &prelude.smaller_external_lower_polygons_series[..],
            LowerFlowRoute::External => &prelude.external_lower_polygons_series[..],
            LowerFlowRoute::Internal => &prelude.lower_polygons_series[..],
        };
        assert!(std::ptr::eq(
            object.lower_series(record_index, seed.route),
            expected
        ));
        pending.extend(&seed.children);
    }
}

fn seed_summary(roots: &[TraversalSeed]) -> (usize, i128) {
    let mut count = 0;
    let mut checksum = 0_i128;
    let mut pending = roots.iter().rev().collect::<Vec<_>>();
    while let Some(seed) = pending.pop() {
        count += 1;
        checksum = checksum
            .wrapping_mul(37)
            .wrapping_add(i128::from(seed.depth))
            .wrapping_add(i128::from(seed.width.to_bits()))
            .wrapping_add(i128::from(seed.mm3_per_mm.to_bits()));
        for point in seed.polygon.points() {
            checksum = checksum
                .wrapping_mul(31)
                .wrapping_add(i128::from(point.x()) + 7 * i128::from(point.y()));
        }
        pending.extend(seed.children.iter().rev());
    }
    (count, checksum)
}
