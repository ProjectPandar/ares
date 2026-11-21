//! PrintObject.cpp:1169-1256,3357-3364: octree sets and original bridge input.
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

pub(in crate::project_slice) struct ObjectSpacing {
    adaptive: Vec<f64>,
    support: Vec<f64>,
    pub(super) body_trees: std::sync::OnceLock<super::body_trees::BodyTrees>,
}

impl ObjectSpacing {
    pub(in crate::project_slice) fn for_pattern(
        &self,
        pattern: ProcessInfillPattern,
        region_id: usize,
    ) -> f64 {
        match pattern {
            ProcessInfillPattern::AdaptiveCubic => self.adaptive[region_id],
            ProcessInfillPattern::SupportCubic => self.support[region_id],
            _ => unreachable!("spacing is only defined for adaptive patterns"),
        }
    }

    pub(super) fn active_sets(&self) -> impl Iterator<Item = (usize, (u64, bool))> + '_ {
        [(&self.adaptive, false), (&self.support, true)]
            .into_iter()
            .flat_map(|(regions, support)| {
                regions
                    .iter()
                    .enumerate()
                    .filter_map(move |(region, &spacing)| {
                        (spacing > 0.0).then_some((region, (spacing.to_bits(), support)))
                    })
            })
    }
}

pub(in crate::project_slice) fn object_spacings(
    traversal: &PreparedPostClassicTraversal,
    prelude: &PostPerimeterInputPrintObject,
    records: &[Option<PreparedSurfaceTypeRecord>],
    layer_count: usize,
) -> ObjectSpacing {
    let active = active_region_ids(prelude, records, layer_count);
    let (_, _, regions) = prelude.as_parts().0.as_parts().0.as_parts();
    let region_options = regions
        .iter()
        .map(|region| region.as_parts().1)
        .collect::<Vec<_>>();
    let nozzles = &traversal.resolved.views.full.project.print.nozzle_diameter;
    // FillAdaptive.cpp:304-323 / PrintObject.cpp:1169-1189.
    let spacings = |pattern| {
        region_options
            .iter()
            .enumerate()
            .map(|(region_id, region)| {
                if active.contains(&region_id)
                    && region.sparse_infill_pattern == pattern
                    && region.sparse_infill_density.0 > 0.0
                {
                    super::line_spacing::configured(region, nozzles, region.fill_multiline.0)
                } else {
                    0.0
                }
            })
            .collect()
    };
    ObjectSpacing {
        adaptive: spacings(ProcessInfillPattern::AdaptiveCubic),
        support: spacings(ProcessInfillPattern::SupportCubic),
        body_trees: std::sync::OnceLock::new(),
    }
}

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

pub(in crate::project_slice) struct SupportPreparation<'a> {
    pub(in crate::project_slice) traversal: &'a PreparedPostClassicTraversal,
    pub(in crate::project_slice) object_index: usize,
    pub(in crate::project_slice) candidates: &'a BridgeCandidateObject,
    pub(in crate::project_slice) horizontal: &'a PreparedSurfaceTypeObject,
    pub(in crate::project_slice) prelude: &'a PostPerimeterInputPrintObject,
    pub(in crate::project_slice) spacing: &'a ObjectSpacing,
}

pub(in crate::project_slice) fn prepare_octrees(
    context: SupportPreparation<'_>,
) -> Result<(), crate::SliceError> {
    super::body_trees::prepare(&context)?;
    if context.spacing.body_trees.get().is_none() {
        prepare_support(context);
    }
    Ok(())
}

fn prepare_support(context: SupportPreparation<'_>) {
    let SupportPreparation {
        traversal,
        object_index,
        candidates,
        horizontal,
        prelude,
        spacing,
    } = context;
    let (plan, _, _) = prelude.as_parts().0.as_parts().0.as_parts();
    let spacings = spacing
        .support
        .iter()
        .copied()
        .filter(|&spacing| spacing > 0.0)
        .map(f64::to_bits)
        .collect::<std::collections::BTreeSet<_>>();
    if spacings.is_empty() {
        return;
    }
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
    for bits in spacings {
        traversal
            .adaptive_octrees
            .get_or_build((object_index, bits, OctreeKind::Support), || {
                super::build_object_octree(
                    traversal,
                    object_index,
                    f64::from_bits(bits),
                    OctreeKind::Support,
                    &triangles,
                )
            });
    }
}
