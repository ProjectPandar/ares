mod classify;
pub(in crate::project_slice) mod types;

pub(in crate::project_slice) use types::{
    ClassicTraversalRecord, InactiveOverhangReverse, LowerFlowRoute, PendingExtrusionRole,
    PendingLoopRole, PendingPathBranch, PostClassicTraversalPrintObject,
    PreparedPostClassicTraversal, PreparedTraversalSurface, TraversalSeed,
};

use super::hierarchy::{PostClassicHierarchyPrintObject, PreparedPostClassicHierarchy};
use classify::classify_roots;
use types::{ClassicTraversalRecord as Record, RouteFlows};

pub(super) fn finish(prepared: PreparedPostClassicHierarchy) -> PreparedPostClassicTraversal {
    let simplification_tolerance = prepared.resolved.views.full.process.print.resolution.0;
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
            prepare_object(predecessor, raft_layers, simplification_tolerance)
        })
        .collect::<Vec<_>>();
    let PreparedPostClassicHierarchy {
        project,
        resolved,
        config_block,
        scale,
        objects: predecessors,
    } = prepared;
    PreparedPostClassicTraversal {
        project,
        resolved,
        config_block,
        scale,
        objects: predecessors
            .into_iter()
            .zip(objects)
            .map(|(predecessor, records)| PostClassicTraversalPrintObject {
                predecessor,
                records,
            })
            .collect(),
        #[cfg(test)]
        drop_probe: types::TraversalDropProbe::new(),
    }
}

fn prepare_object(
    hierarchy: &PostClassicHierarchyPrintObject,
    raft_layers: i32,
    simplification_tolerance: f64,
) -> Vec<Option<ClassicTraversalRecord>> {
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
        .map(|((hierarchy_record, prelude_record), input)| {
            match (hierarchy_record, prelude_record, input) {
                (Some(hierarchy_record), Some(prelude), Some(input)) => {
                    let region = input_object.region_options(input);
                    let odd_layer = input.layer_id % 2 == 1;
                    let flows = RouteFlows {
                        perimeter: input.perimeter_flow,
                        external: input.ext_perimeter_flow,
                        smaller_external: prelude.smaller_external_flow,
                    };
                    let surfaces = hierarchy_record
                        .surfaces
                        .iter()
                        .map(|surface| PreparedTraversalSurface {
                            source_index: surface.source_index,
                            roots: classify_roots(&surface.roots, flows),
                        })
                        .collect();
                    Some(Record {
                        surfaces,
                        layer_id: input.layer_id,
                        layer_height: input.layer_height,
                        slice_z: plan.layers[input.planned_layer_index].slice_z,
                        fuzzy_skin: crate::perimeters::FuzzySkinConfig::from_region(region),
                        simplification_tolerance,
                        overhang_flow: input.overhang_flow,
                        branch: PendingPathBranch::from_operands(
                            region.detect_overhang_wall.0,
                            input.layer_id,
                            raft_layers,
                        ),
                        overhang_reverse: InactiveOverhangReverse {
                            configured: region.overhang_reverse.0,
                            odd_layer,
                            active: region.overhang_reverse.0 && odd_layer,
                        },
                    })
                }
                (None, None, None) => None,
                _ => unreachable!("O5 record slots remain aligned with O4 and O1"),
            }
        })
        .collect()
}

impl PostClassicTraversalPrintObject {
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

    pub(in crate::project_slice) fn lower_slices(
        &self,
        record_index: usize,
    ) -> Option<&[crate::geometry::ExPolygon]> {
        let prelude = &self.predecessor.predecessor.predecessor.predecessor;
        let input = prelude.object.records[record_index].as_ref()?;
        prelude.object.lower_slices(input)
    }

    /// Current layer's merged slices (`Layer::lslices`), the base of the
    /// avoid-crossing boundary (`AvoidCrossingPerimeters.cpp:1099-1134`).
    pub(in crate::project_slice) fn slices(
        &self,
        record_index: usize,
    ) -> Option<&[crate::geometry::ExPolygon]> {
        let prelude = &self.predecessor.predecessor.predecessor.predecessor;
        prelude
            .object
            .records
            .get(record_index)
            .and_then(|record| record.as_ref())
            .map(|record| prelude.object.current_slices(record))
    }

    /// Perimeter flow spacing of the record's region in millimetres
    /// (`get_perimeter_spacing`, `AvoidCrossingPerimeters.cpp:499-512`).
    pub(in crate::project_slice) fn perimeter_spacing(&self, record_index: usize) -> Option<f32> {
        let prelude = &self.predecessor.predecessor.predecessor.predecessor;
        prelude
            .object
            .records
            .get(record_index)
            .and_then(|record| record.as_ref())
            .map(|record| record.perimeter_flow.spacing)
    }

    /// Top fill surfaces of the record's region-layer; the avoid-crossing
    /// boundary subtracts their inset from the slice union
    /// (`AvoidCrossingPerimeters.cpp:1122-1132`).
    pub(in crate::project_slice) fn top_surfaces(
        &self,
        record_index: usize,
    ) -> Option<Vec<&crate::geometry::ExPolygon>> {
        let prelude = &self.predecessor.predecessor.predecessor.predecessor;
        let record = prelude.object.records.get(record_index)?.as_ref()?;
        Some(
            prelude
                .object
                .current_surfaces(record)
                .iter()
                .filter(|surface| {
                    matches!(
                        surface.as_parts().0,
                        crate::project_slice::region_slices::RegionSurfaceKind::Top
                    )
                })
                .map(|surface| surface.as_parts().1)
                .collect(),
        )
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

#[cfg(test)]
mod tests;
