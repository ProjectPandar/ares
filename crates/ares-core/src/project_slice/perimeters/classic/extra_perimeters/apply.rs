//! `PerimeterGenerator::apply_extra_perimeters` and classic activation operands.

use crate::{
    geometry::{CoordinateScale, Polygon, Polyline},
    project_slice::perimeters::{
        classic::{
            entity_collections::ExtrusionEntity,
            gap_extrusion::PreparedPostClassicGapExtrusion,
            materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
            traversal::PostClassicTraversalPrintObject,
        },
        types::Flow,
    },
};

pub(in crate::project_slice::perimeters) struct Context<'a> {
    pub(in crate::project_slice::perimeters) lower: &'a [Polygon],
    pub(in crate::project_slice::perimeters) perimeter_count: i32,
    pub(in crate::project_slice::perimeters) flow: Flow,
    pub(in crate::project_slice::perimeters) scale: CoordinateScale,
}

pub(in crate::project_slice::perimeters::classic) fn context<'a>(
    prepared: &'a PreparedPostClassicGapExtrusion,
    traversal: &'a PostClassicTraversalPrintObject,
    index: usize,
) -> Option<Context<'a>> {
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    let input = &prelude.object;
    let record = input.records[index].as_ref().unwrap();
    let region = input.region_options(record);
    if record.spiral_mode
        || !region.extra_perimeters_on_overhangs.0
        || !region.detect_overhang_wall.0
        || region.wall_loops.0 <= 0
    {
        return None;
    }
    traversal.lower_slices(index)?;
    let options = &prepared
        .predecessor
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == input.identity().0)
        .unwrap()
        .object;
    if i64::try_from(record.layer_id).unwrap() <= i64::from(options.raft_layers.0) {
        return None;
    }
    Some(Context {
        lower: &prelude.records[index]
            .as_ref()
            .unwrap()
            .lower_slices_polygons,
        perimeter_count: region.wall_loops.0,
        flow: traversal.records[index].as_ref().unwrap().overhang_flow,
        scale: prepared.predecessor.scale,
    })
}

impl Context<'_> {
    pub(in crate::project_slice::perimeters) fn materialize(
        &self,
        paths: Vec<Polyline>,
    ) -> Vec<ExtrusionEntity> {
        paths
            .into_iter()
            .map(|path| {
                ExtrusionEntity::Path(ExtrusionPath {
                    z_contoured: false,
                    polyline: Polyline3 {
                        points: path
                            .into_points()
                            .into_iter()
                            .map(|p| Point3 {
                                x: p.x(),
                                y: p.y(),
                                z: 0,
                            })
                            .collect(),
                        fitting: Vec::new(),
                    },
                    role: ExtrusionRole::OverhangPerimeter,
                    can_reverse: true,
                    mm3_per_mm: self.flow.mm3_per_mm,
                    width: self.flow.width,
                    height: self.flow.height,
                })
            })
            .collect()
    }
}
