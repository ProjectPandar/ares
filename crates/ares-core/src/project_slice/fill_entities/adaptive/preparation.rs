//! PrintObject.cpp:972-1007/2728-2735: original bridge input and shared tree.
use crate::{
    ProcessInfillPattern,
    fill::adaptive::OctreeKind,
    geometry::triangulate_expolygon_3d,
    project_slice::{
        perimeters::{
            classic::traversal::types::PreparedPostClassicTraversal,
            types::{PostPerimeterInputPrintObject, RegionLayerIndex},
        },
        prepare_infill::{
            bridge_over_infill::BridgeCandidateObject,
            surface_type_detection::types::{PreparedSurfaceTypeObject, PreparedSurfaceTypeRecord},
        },
    },
};

pub(in crate::project_slice) fn active_region_ids(
    prelude: &PostPerimeterInputPrintObject,
    records: &[Option<PreparedSurfaceTypeRecord>],
    layer_count: usize,
) -> Vec<usize> {
    let (_, _, regions) = prelude.as_parts().0.as_parts().0.as_parts();
    regions
        .iter()
        .enumerate()
        .filter_map(|(region_index, _)| {
            (0..layer_count)
                .any(|layer_index| {
                    let slot = prelude.region_layer_record_index(RegionLayerIndex {
                        region_index,
                        layer_index,
                    });
                    records[slot]
                        .as_ref()
                        .is_some_and(|record| !record.fill_surfaces.is_empty())
                })
                .then_some(region_index)
        })
        .collect()
}

pub(in crate::project_slice) fn prepare_support(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    candidates: &BridgeCandidateObject,
    horizontal: &PreparedSurfaceTypeObject,
    prelude: &PostPerimeterInputPrintObject,
) {
    let (plan, _, regions) = prelude.as_parts().0.as_parts().0.as_parts();
    let nozzles = &traversal.resolved.views.full.project.print.nozzle_diameter;
    let active_region_ids = active_region_ids(prelude, &horizontal.records, plan.layers.len());
    let mut contributing = regions.iter().enumerate().filter(|(index, region)| {
        let options = region.as_parts().1;
        options.sparse_infill_pattern == ProcessInfillPattern::SupportCubic
            && options.sparse_infill_density.0 > 0.0
            && active_region_ids.contains(index)
    });
    if contributing.next().is_none() {
        return;
    }
    let resolved = traversal
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == plan.source_object_index)
        .expect("support object retains resolved regions");
    let spacing = super::line_spacing::object_configured(
        &resolved.all_regions,
        nozzles,
        ProcessInfillPattern::SupportCubic,
        &active_region_ids,
    );
    let mut triangles = Vec::new();
    // Source map/layer order and each candidate vector's original order.
    for candidate in candidates.surfaces_by_layer.values().flatten() {
        let source = candidate.source;
        let slot = prelude.region_layer_record_index(RegionLayerIndex {
            region_index: source.region_index,
            layer_index: source.layer_index,
        });
        let record = horizontal.records[slot]
            .as_ref()
            .expect("original bridge candidate retains its source record");
        let surface = &record.fill_surfaces[source.surface_index];
        let layer = &plan.layers[source.layer_index];
        // The upstream pair stores float bottom_z before Tesselate.cpp:192-195.
        let z = f64::from((layer.print_z - layer.height) as f32);
        triangles.extend(triangulate_expolygon_3d(
            surface.as_parts().1,
            z,
            false,
            traversal.scale,
        ));
    }
    traversal.adaptive_octrees.get_or_build(
        (object_index, spacing.to_bits(), OctreeKind::Support),
        || {
            super::build_object_octree(
                traversal,
                object_index,
                spacing,
                OctreeKind::Support,
                &triangles,
            )
        },
    );
}
