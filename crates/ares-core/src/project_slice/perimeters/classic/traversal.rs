mod classify;
pub(in crate::project_slice) mod types;

pub(in crate::project_slice) use types::{
    ClassicTraversalRecord, LowerFlowRoute, PendingExtrusionRole, PendingLoopRole,
    PendingPathBranch, PostClassicTraversalPrintObject, PreparedPostClassicTraversal,
    PreparedTraversalSurface, TraversalSeed,
};

use super::hierarchy::{PostClassicHierarchyPrintObject, PreparedPostClassicHierarchy};
use classify::classify_roots;
use types::{ClassicTraversalRecord as Record, RouteFlows};

pub(super) fn finish(
    prepared: PreparedPostClassicHierarchy,
) -> Result<PreparedPostClassicTraversal, crate::SliceError> {
    let objects = prepared
        .objects
        .iter()
        .map(|predecessor| {
            let (source_object_index, _) = predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .identity();
            let raft_layers = prepared
                .resolved
                .objects
                .iter()
                .find(|object| object.source_object_index == source_object_index)
                .expect("O5 object retains its resolved source")
                .object
                .raft_layers
                .0;
            prepare_object(predecessor, raft_layers, prepared.scale)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let PreparedPostClassicHierarchy {
        project,
        resolved,
        config_block,
        scale,
        objects: predecessors,
    } = prepared;
    Ok(PreparedPostClassicTraversal {
        project,
        resolved,
        config_block,
        scale,
        adaptive_octrees: crate::fill::adaptive::AdaptiveOctreeCache::new(),
        objects: predecessors
            .into_iter()
            .zip(objects)
            .map(|(predecessor, records)| PostClassicTraversalPrintObject {
                predecessor,
                records,
            })
            .collect(),
    })
}

fn prepare_object(
    hierarchy: &PostClassicHierarchyPrintObject,
    raft_layers: i32,
    scale: crate::geometry::CoordinateScale,
) -> Result<Vec<Option<ClassicTraversalRecord>>, crate::SliceError> {
    let onion = &hierarchy.predecessor;
    let top_split = &onion.predecessor;
    let prelude_object = &top_split.predecessor;
    let input_object = &prelude_object.object;
    let (post_regions, _) = input_object.object.as_parts();
    let (plan, _, _) = post_regions.as_parts();
    hierarchy
        .records
        .iter()
        .zip(&prelude_object.records)
        .zip(input_object.as_parts().1)
        .enumerate()
        .map(
            |(record_index, ((hierarchy_record, prelude_record), input))| {
                match (hierarchy_record, prelude_record, input) {
                    (Some(hierarchy_record), Some(prelude), Some(input)) => {
                        let region = input_object.region_options(input);
                        let flows = RouteFlows {
                            perimeter: input.perimeter_flow,
                            external: input.ext_perimeter_flow,
                            smaller_external: prelude.smaller_external_flow,
                        };
                        let onion_surfaces = onion.records[record_index]
                            .as_ref()
                            .map(|record| record.surfaces.as_slice())
                            .unwrap_or(&[]);
                        let surfaces = hierarchy_record
                            .surfaces
                            .iter()
                            .map(|surface| PreparedTraversalSurface {
                                source_index: surface.source_index,
                                roots: classify_roots(&surface.roots, flows),
                                // `detect_thin_wall` walls carried from the onion
                                // surface at the same `source_index`.
                                thin_walls: onion_surfaces
                                    .iter()
                                    .find(|o| o.source_index == surface.source_index)
                                    .map(|o| o.thin_walls.clone())
                                    .unwrap_or_default(),
                            })
                            .collect();
                        Ok(Some(Record {
                            surfaces,
                            layer_id: input.layer_id,
                            layer_height: input.layer_height,
                            slice_z: plan.layers[input.planned_layer_index].slice_z,
                            fuzzy_skin: super::super::arachne::fuzzy::Regions::prepare(
                                input_object,
                                input,
                                scale,
                            )?,
                            overhang_flow: input.overhang_flow,
                            ext_perimeter_flow: input.ext_perimeter_flow,
                            scale,
                            branch: PendingPathBranch::from_operands(
                                region.detect_overhang_wall.0,
                                input.layer_id,
                                raft_layers,
                            ),
                        }))
                    }
                    (None, None, None) => Ok(None),
                    _ => unreachable!("O5 record slots remain aligned with O4 and O1"),
                }
            },
        )
        .collect()
}

impl PostClassicTraversalPrintObject {
    pub(in crate::project_slice) fn physical_layers(
        &self,
    ) -> &[crate::project_slice::layers::PlannedLayer] {
        let input = &self.predecessor.predecessor.predecessor.predecessor.object;
        &input.object.as_parts().0.plan.layers
    }

    pub(in crate::project_slice) fn slicing_parameters(
        &self,
    ) -> &crate::project_slice::parameters::SlicingParameters {
        let input = &self.predecessor.predecessor.predecessor.predecessor.object;
        &input.object.as_parts().0.plan.slicing_parameters
    }

    pub(in crate::project_slice) fn lower_series(
        &self,
        record_index: usize,
        route: LowerFlowRoute,
    ) -> &[Vec<crate::geometry::Polygon>] {
        let prelude = self.predecessor.predecessor.predecessor.predecessor.records[record_index]
            .as_ref()
            .expect("a traversal seed has an aligned prelude record");
        match route {
            LowerFlowRoute::SmallerExternal => &prelude.smaller_external_lower_polygons_series,
            LowerFlowRoute::External => &prelude.external_lower_polygons_series,
            LowerFlowRoute::Internal => &prelude.lower_polygons_series,
        }
    }

    /// `ExtrusionProcessor.hpp:305-311::prepare_for_new_layer` consumes
    /// the preceding physical layer's merged `Layer::lslices`, even when
    /// modifiers have emptied the base region's record slots.
    pub(in crate::project_slice) fn lower_slices(
        &self,
        layer_index: usize,
    ) -> Option<&[crate::geometry::ExPolygon]> {
        self.slices(layer_index.checked_sub(1)?)
    }

    /// Current layer's merged slices (`Layer::lslices`), the base of the
    /// avoid-crossing boundary (`AvoidCrossingPerimeters.cpp:1099-1134`).
    pub(in crate::project_slice) fn slices(
        &self,
        layer_index: usize,
    ) -> Option<&[crate::geometry::ExPolygon]> {
        let prelude = &self.predecessor.predecessor.predecessor.predecessor;
        prelude
            .object
            .records
            .iter()
            .skip(layer_index)
            .step_by(self.physical_layers().len())
            .flatten()
            .next()
            .map(|record| prelude.object.current_slices(record))
    }

    /// Perimeter flow spacing of the record's region in millimetres
    /// (`get_perimeter_spacing`, `AvoidCrossingPerimeters.cpp:499-512`).
    pub(in crate::project_slice) fn perimeter_spacing(&self, layer_index: usize) -> Option<f32> {
        let prelude = &self.predecessor.predecessor.predecessor.predecessor;
        let mut total = 0.0;
        let mut count = 0;
        for record in prelude
            .object
            .records
            .iter()
            .skip(layer_index)
            .step_by(self.physical_layers().len())
            .flatten()
        {
            total += record.perimeter_flow.spacing;
            count += 1;
        }
        (count != 0).then(|| total / count as f32)
    }

    /// External perimeter flow width of the record's region in millimetres
    /// (`get_external_perimeter_width`,
    /// `AvoidCrossingPerimeters.cpp:531-545`).
    pub(in crate::project_slice) fn external_perimeter_width(
        &self,
        layer_index: usize,
    ) -> Option<f32> {
        let prelude = &self.predecessor.predecessor.predecessor.predecessor;
        let mut total = 0.0;
        let mut count = 0;
        for record in prelude
            .object
            .records
            .iter()
            .skip(layer_index)
            .step_by(self.physical_layers().len())
            .flatten()
        {
            total += record.ext_perimeter_flow.width;
            count += 1;
        }
        (count != 0).then(|| total / count as f32)
    }

    pub(in crate::project_slice) fn wall_direction(
        &self,
        record_index: usize,
    ) -> crate::ProcessWallDirection {
        let prelude = &self.predecessor.predecessor.predecessor.predecessor;
        let input = prelude.object.records[record_index]
            .as_ref()
            .expect("an O9 collection has an aligned perimeter input");
        prelude.object.region_options(input).wall_direction
    }
}
