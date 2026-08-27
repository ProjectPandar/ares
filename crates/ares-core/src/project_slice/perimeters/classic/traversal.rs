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
            prepare_object(predecessor, raft_layers)
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
) -> Vec<Option<ClassicTraversalRecord>> {
    let onion = &hierarchy.predecessor;
    let top_split = &onion.predecessor;
    let prelude_object = &top_split.predecessor;
    let input_object = &prelude_object.object;
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
                        layer_height: input.layer_height,
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
