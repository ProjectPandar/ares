//! `GCode/PrintExtents.cpp`: priming collision checks over printable copies.
use super::{
    super::{layers::Instance, skirt::SkirtPlan},
    Data,
};
use crate::{
    geometry::CoordinateScale,
    project_slice::{
        fill_entities::FillExtrusionEntity,
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::{
            entity_collections::ExtrusionEntity, gap_extrusion::GapFillEntity,
            materialize::ExtrusionPath, traversal::PreparedPostClassicTraversal,
        },
    },
};

pub(in crate::project_slice::gcode_emit) struct Print<'a> {
    pub(in crate::project_slice::gcode_emit) traversal: &'a PreparedPostClassicTraversal,
    pub(in crate::project_slice::gcode_emit) objects: &'a [Vec<OrderedExtrusionLayer>],
    pub(in crate::project_slice::gcode_emit) instances: &'a [Vec<Instance>],
    pub(in crate::project_slice::gcode_emit) skirt: Option<&'a SkirtPlan>,
    pub(in crate::project_slice::gcode_emit) max_print_z: f64,
}

#[derive(Default)]
struct Extents(Option<[[f64; 2]; 2]>);
impl Extents {
    fn point(&mut self, point: [f64; 2]) {
        if let Some([min, max]) = &mut self.0 {
            for axis in 0..2 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        } else {
            self.0 = Some([point, point]);
        }
    }
    fn path(
        &mut self,
        points: impl Iterator<Item = (i64, i64)>,
        width: f32,
        scale: CoordinateScale,
    ) {
        let radius = (0.5 * f64::from(width) / scale.factor()) as i64;
        for (x, y) in points {
            self.point([scale.unscale(x - radius), scale.unscale(y - radius)]);
            self.point([scale.unscale(x + radius), scale.unscale(y + radius)]);
        }
    }
    fn path3(&mut self, path: &ExtrusionPath, scale: CoordinateScale) {
        self.path(
            path.polyline.points.iter().map(|point| (point.x, point.y)),
            path.width,
            scale,
        );
    }
    fn gap(&mut self, gap: &GapFillEntity, scale: CoordinateScale) {
        match gap {
            GapFillEntity::Path(path) => self.path3(path, scale),
            GapFillEntity::Loop(paths) => {
                for path in paths {
                    self.path3(path, scale);
                }
            }
        }
    }
    fn fill(&mut self, fill: &FillExtrusionEntity, scale: CoordinateScale) {
        match fill {
            FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => self.path(
                path.polyline.points().iter().map(|p| (p.x(), p.y())),
                path.width,
                scale,
            ),
            FillExtrusionEntity::VariableWidth(gap) => self.gap(gap, scale),
        }
    }
    fn entity(&mut self, entity: &IslandPrintEntity, scale: CoordinateScale) {
        match entity {
            IslandPrintEntity::Perimeter(collection) => {
                for entity in &collection.entities {
                    self.perimeter(entity, scale);
                }
            }
            IslandPrintEntity::Fill(fill) => self.fill(fill, scale),
            IslandPrintEntity::FillCollection(collection) => {
                for fill in &collection.entities {
                    self.fill(fill, scale);
                }
            }
            IslandPrintEntity::Thin(gap) => self.gap(gap, scale),
        }
    }
    fn perimeter(&mut self, entity: &ExtrusionEntity, scale: CoordinateScale) {
        match entity {
            ExtrusionEntity::Loop(loop_) => {
                for path in &loop_.extrusion_loop.paths {
                    self.path3(path, scale);
                }
            }
            ExtrusionEntity::MultiPath(multi) => {
                for path in &multi.paths {
                    self.path3(path, scale);
                }
            }
            ExtrusionEntity::Path(path) => self.path3(path, scale),
        }
    }
    fn merge_copies(&mut self, local: Self, copies: &[Instance], scale: CoordinateScale) {
        let Some(bounds) = local.0 else {
            return;
        };
        for copy in copies {
            for [x, y] in bounds {
                self.point([
                    x + scale.unscale(copy.shift.x()),
                    y + scale.unscale(copy.shift.y()),
                ]);
            }
        }
    }
    fn tower(
        &mut self,
        result: &super::result::ToolChangeResult,
        transform: impl Fn([f32; 2]) -> [f64; 2],
    ) {
        for pair in result
            .extrusions
            .windows(2)
            .filter(|pair| pair[1].width > 0.0)
        {
            let radius = 0.5 * f64::from(pair[1].width);
            for extrusion in pair {
                let [x, y] = transform(extrusion.position);
                self.point([x - radius, y - radius]);
                self.point([x + radius, y + radius]);
            }
        }
    }
}

impl Data {
    pub(in crate::project_slice::gcode_emit) fn priming_overlaps(&self, print: Print<'_>) -> bool {
        let scale = print.traversal.scale;
        let mut occupied = Extents::default();
        if let Some(skirt) = print.skirt {
            for (points, width) in skirt.extrusion_paths() {
                occupied.path(
                    points.iter().map(|point| (point.x(), point.y())),
                    width,
                    scale,
                );
            }
        }
        for ((layers, object), copies) in print
            .objects
            .iter()
            .zip(&print.traversal.objects)
            .zip(print.instances)
        {
            let mut local = Extents::default();
            for entity in layers
                .iter()
                .zip(object.physical_layers())
                .take_while(|(_, physical)| physical.print_z <= print.max_print_z)
                .flat_map(|(layer, _)| &layer.islands)
                .flat_map(|island| &island.entities)
            {
                local.entity(entity, scale);
            }
            occupied.merge_copies(local, copies, scale);
        }
        let settings = &print.traversal.resolved.views.full;
        let (sin, cos) =
            (settings.process.print.wipe_tower_rotation_angle.0 * std::f64::consts::PI / 180.0)
                .sin_cos();
        // `PrintExtents.cpp:152-154`: the plate's stored position plus the plate origin.
        let [x, y] = super::plate_position(print.traversal);
        let origin = print.traversal.project.plate_origin();
        let position = [x + origin.0, y + origin.1];
        // `PrintExtents.cpp:155-158`: the composed transform translates by
        // `position + R * rib_offset`.
        let [rib_x, rib_y] = self.rib_offset.map(f64::from);
        let translation = [
            position[0] + (cos * rib_x - sin * rib_y),
            position[1] + (sin * rib_x + cos * rib_y),
        ];
        for result in self
            .changes
            .iter()
            .take_while(|row| {
                row.first()
                    .is_none_or(|result| f64::from(result.print_z) <= print.max_print_z)
            })
            .flatten()
        {
            occupied.tower(result, |[x, y]| {
                let (x, y) = (f64::from(x), f64::from(y));
                [
                    cos * x - sin * y + translation[0],
                    sin * x + cos * y + translation[1],
                ]
            });
        }
        let mut priming = Extents::default();
        for result in &self.priming {
            priming.tower(result, |point| point.map(f64::from));
        }
        match (priming.0, occupied.0) {
            (Some([min, max]), Some([other_min, other_max])) => (0..2).all(|axis| {
                min[axis] - 0.5 <= other_max[axis] && max[axis] + 0.5 >= other_min[axis]
            }),
            _ => false,
        }
    }
}
