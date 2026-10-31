//! Auto lift overhang cache (`PrintObject.cpp:818–848`) and recent-layer
//! query (`GCode.cpp:7538–7605`, `Print.cpp:594–614`).

use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, Point, Polygon, Polyline,
    intersection_open_polylines,
};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::{FloatOrPercent, ZHopType};

mod detect;

#[derive(Default)]
pub(in crate::project_slice::gcode_emit) struct AutoLift {
    objects: Vec<Object>,
    maximum_hop: f32,
}

struct Object {
    shifts: Vec<Point>,
    layers: Vec<Layer>,
}

struct Layer {
    print_z: f64,
    bounds: Option<BoundingBox>,
    polygons: Vec<Polygon>,
}

impl AutoLift {
    pub(in crate::project_slice::gcode_emit) fn prepare(
        traversal: &PreparedPostClassicTraversal,
    ) -> Result<Self, ClipperError> {
        let runtime = &traversal.resolved.views.runtime_gcode;
        if !runtime.z_hop_types.0.contains(&ZHopType::Auto) {
            return Ok(Self::default());
        }
        let scale = traversal.scale;
        let nozzle = traversal
            .resolved
            .views
            .full
            .project
            .print
            .nozzle_diameter
            .0[0]
            .0;
        let mut objects = Vec::with_capacity(traversal.objects.len());
        for object in &traversal.objects {
            let input = &object
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object;
            let (regions, slices) = input.object.as_parts();
            let (plan, _, _) = regions.as_parts();
            let resolved = traversal
                .resolved
                .objects
                .iter()
                .find(|resolved| resolved.source_object_index == plan.source_object_index)
                .unwrap();
            let options = &resolved.object;
            let line_width = match options.line_width {
                FloatOrPercent::Float(value) => value,
                FloatOrPercent::Percent(value) => value.0 * nozzle / 100.0,
            };
            let layers = detect::layers(slices, plan, line_width, scale)?;
            let project_object = &traversal.project.objects()[plan.source_object_index];
            let center = crate::project_slice::centering::center(
                project_object,
                &resolved.print_objects[plan.transform_index],
                scale,
            )
            .expect("sliced object has validated raw bounds");
            objects.push(Object {
                shifts: resolved.print_objects[plan.transform_index]
                    .instance_indices
                    .iter()
                    .map(|&index| {
                        super::super::footprint::instance_shift(
                            &project_object.instances()[index],
                            center,
                            scale,
                        )
                    })
                    .collect(),
                layers,
            });
        }
        Ok(Self {
            objects,
            maximum_hop: runtime
                .z_hop
                .0
                .iter()
                .fold(0.0_f32, |maximum, value| maximum.max(value.0 as f32)),
        })
    }

    pub(in crate::project_slice::gcode_emit) fn crosses_overhang(
        &self,
        state: &super::EmitState,
        [start, mut end]: [Point; 2],
        scale: CoordinateScale,
    ) -> bool {
        let dx = (end.x() - start.x()) as f64;
        let dy = (end.y() - start.y()) as f64;
        let length = (dx * dx + dy * dy).sqrt();
        let threshold = (f64::from(self.maximum_hop)
            / state.options.travel_slope_radians.tan()
            / scale.factor()) as f32;
        if length > f64::from(threshold) {
            let ratio = f64::from(threshold) / length;
            end = Point::new(
                start.x() + (dx * ratio) as i64,
                start.y() + (dy * ratio) as i64,
            );
        }
        let offset = Point::new(
            scale.checked_scale(state.origin.0).unwrap(),
            scale.checked_scale(state.origin.1).unwrap(),
        );
        let translate = |point: Point| Point::new(point.x() + offset.x(), point.y() + offset.y());
        let (start, end) = (translate(start), translate(end));
        let upper = state.active_layer_print_z as f32;
        let lower = (upper - 0.4_f32).max(0.0);
        self.objects.iter().any(|object| {
            object.crosses(
                [start, end],
                (f64::from(lower), f64::from(upper) + 1e-4),
                scale,
            )
        })
    }
}

impl Object {
    fn crosses(
        &self,
        [start, end]: [Point; 2],
        (lower, upper): (f64, f64),
        scale: CoordinateScale,
    ) -> bool {
        let first = self.layers.partition_point(|layer| layer.print_z < lower);
        let last = self.layers.partition_point(|layer| layer.print_z <= upper);
        let layers = &self.layers[first..last];
        let Some(mut object_bounds) =
            layers
                .iter()
                .filter_map(|layer| layer.bounds)
                .reduce(|mut bounds, next| {
                    bounds.merge_point(next.min());
                    bounds.merge_point(next.max());
                    bounds
                })
        else {
            return false;
        };
        object_bounds.offset(scale.checked_scale(1e-4).unwrap());
        let min = object_bounds.min();
        let max = object_bounds.max();
        self.shifts.iter().any(|shift| {
            let translate = |point: Point| Point::new(point.x() - shift.x(), point.y() - shift.y());
            let (start, end) = (translate(start), translate(end));
            let bounds = BoundingBox::new(
                Point::new(start.x().min(end.x()), start.y().min(end.y())),
                Point::new(start.x().max(end.x()), start.y().max(end.y())),
            );
            if !overlaps(bounds, object_bounds) {
                return false;
            }
            let travel = Polyline::new(vec![start, end]);
            let rectangle = Polygon::new(vec![
                min,
                Point::new(max.x(), min.y()),
                max,
                Point::new(min.x(), max.y()),
            ]);
            if intersection_open_polylines(
                std::slice::from_ref(&travel),
                std::slice::from_ref(&rectangle),
            )
            .expect("validated object bounds clip successfully")
            .is_empty()
            {
                return false;
            }
            layers.iter().any(|layer| layer.crosses(bounds, &travel))
        })
    }
}

impl Layer {
    fn crosses(&self, bounds: BoundingBox, travel: &Polyline) -> bool {
        self.bounds.is_some_and(|candidate| {
            overlaps(bounds, candidate)
                && !intersection_open_polylines(std::slice::from_ref(travel), &self.polygons)
                    .expect("validated slice and travel coordinates clip successfully")
                    .is_empty()
        })
    }
}

fn overlaps(left: BoundingBox, right: BoundingBox) -> bool {
    left.min().x() <= right.max().x()
        && left.max().x() >= right.min().x()
        && left.min().y() <= right.max().y()
        && left.max().y() >= right.min().y()
}
