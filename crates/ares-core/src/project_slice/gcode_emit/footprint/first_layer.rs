//! Plate-frame bounds (`Print::first_layer_islands`, `GCode.cpp:2930-2939`).

use super::{FirstLayerBounds, instance_shift, object_origin};
use crate::{
    geometry::{CoordinateScale, Point, Polygon},
    project_slice::{
        gcode_emit::{brim::BrimPlan, skirt::SkirtPlan},
        perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

pub(in crate::project_slice::gcode_emit) fn first_layer_bounds(
    traversal: &PreparedPostClassicTraversal,
    skirt: Option<&SkirtPlan>,
    brim: Option<&BrimPlan>,
) -> Option<FirstLayerBounds> {
    let print = &traversal.resolved.views.full.process.print;
    let skirt_height = usize::try_from(print.skirt_height.0).unwrap_or_default();
    let layer_limit = if print.draft_shield == crate::ProcessDraftShield::Enabled {
        usize::MAX
    } else {
        skirt_height
    };
    let scale = traversal.scale;
    let mut bounds = None;
    for (object, resolved) in traversal.objects.iter().zip(&traversal.resolved.objects) {
        let layers = object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .object
            .as_parts()
            .1;
        let source = &traversal.project.objects()[resolved.source_object_index];
        let center = crate::project_slice::raw_intersections::raw_center(source, scale)
            .expect("sliced object has validated raw bounds");
        for instance in source
            .instances()
            .iter()
            .filter(|instance| instance.printable())
        {
            let shift = instance_shift(instance, center, scale);
            let selected_layers = layers
                .iter()
                .take(if skirt.is_some() { layer_limit } else { 1 });
            for polygon in selected_layers.flatten() {
                include_polygon_bounds(polygon.contour(), shift, scale, &mut bounds);
            }
        }
    }
    if let Some(skirt) = skirt {
        include_bounds(skirt.covered_bounds(scale), &mut bounds);
    }
    if let Some(brim) = brim {
        let source =
            &traversal.project.objects()[traversal.resolved.objects[0].source_object_index];
        let (x, y) = object_origin(source, scale);
        include_bounds(
            brim.covered_bounds(scale)
                .map(|(min_x, min_y, max_x, max_y)| (min_x + x, min_y + y, max_x + x, max_y + y)),
            &mut bounds,
        );
    }
    let fallback_distance = if skirt.is_none() && skirt_height > 0 {
        print.skirt_distance.0
    } else {
        0.0
    };
    bounds.map(|(min_x, min_y, max_x, max_y)| {
        (
            min_x - fallback_distance,
            min_y - fallback_distance,
            max_x - min_x + 2.0 * fallback_distance,
            max_y - min_y + 2.0 * fallback_distance,
        )
    })
}

fn include_bounds(additional: Option<FirstLayerBounds>, bounds: &mut Option<FirstLayerBounds>) {
    let Some((min_x, min_y, max_x, max_y)) = additional else {
        return;
    };
    *bounds = Some(match *bounds {
        Some((old_min_x, old_min_y, old_max_x, old_max_y)) => (
            old_min_x.min(min_x),
            old_min_y.min(min_y),
            old_max_x.max(max_x),
            old_max_y.max(max_y),
        ),
        None => (min_x, min_y, max_x, max_y),
    });
}

fn include_polygon_bounds(
    polygon: &Polygon,
    shift: Point,
    scale: CoordinateScale,
    bounds: &mut Option<FirstLayerBounds>,
) {
    for point in polygon.points() {
        let x = scale.unscale(point.x() + shift.x());
        let y = scale.unscale(point.y() + shift.y());
        *bounds = Some(match *bounds {
            Some((min_x, min_y, max_x, max_y)) => {
                (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
            }
            None => (x, y, x, y),
        });
    }
}
