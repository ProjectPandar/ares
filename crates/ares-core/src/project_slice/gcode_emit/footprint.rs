use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::{ProcessDraftShield, ProjectVolumeType};

pub(super) fn first_layer_bounds(
    traversal: &PreparedPostClassicTraversal,
) -> Option<(f64, f64, f64, f64)> {
    let print = &traversal.resolved.views.full.process.print;
    let skirt_height = usize::try_from(print.skirt_height.0).unwrap_or_default();
    let infinite_skirt =
        print.draft_shield == ProcessDraftShield::Enabled && print.skirt_loops.0 > 0;
    let collect_skirt_hull = skirt_height > 0 || infinite_skirt;
    let layer_limit = if infinite_skirt {
        usize::MAX
    } else {
        skirt_height
    };

    let mut bounds = None;
    for object in &traversal.objects {
        let compensation_object = &object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .object;
        let layers = compensation_object.as_parts().1;
        let selected_layers = if collect_skirt_hull {
            layers.iter().take(layer_limit)
        } else {
            layers.iter().take(1)
        };
        for layer in selected_layers {
            for polygon in layer {
                include_polygon_bounds(polygon.contour(), traversal.scale, &mut bounds);
            }
        }
    }

    let (center_x, center_y) = model_center(traversal)?;
    bounds.map(|(mut min_x, mut min_y, mut max_x, mut max_y)| {
        if collect_skirt_hull {
            let distance = print.skirt_distance.0;
            min_x -= distance;
            min_y -= distance;
            max_x += distance;
            max_y += distance;
        }
        (
            min_x + center_x,
            min_y + center_y,
            max_x - min_x,
            max_y - min_y,
        )
    })
}

fn include_polygon_bounds(
    polygon: &crate::geometry::Polygon,
    scale: crate::geometry::CoordinateScale,
    bounds: &mut Option<(f64, f64, f64, f64)>,
) {
    for point in polygon.points() {
        let x = scale.unscale(point.x());
        let y = scale.unscale(point.y());
        *bounds = Some(match *bounds {
            Some((min_x, min_y, max_x, max_y)) => {
                (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
            }
            None => (x, y, x, y),
        });
    }
}

pub(super) fn model_bounds(
    traversal: &PreparedPostClassicTraversal,
) -> Option<(f64, f64, f64, f64)> {
    let object = traversal.project.objects().first()?;
    let instance_transform = object.instances().first()?.transform();
    let mut bounds = None::<(f64, f64, f64, f64)>;
    for volume in object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
    {
        let transform = instance_transform.then(volume.transform());
        for &vertex in volume.mesh().vertices() {
            let point = transform.transform_point(vertex);
            bounds = Some(match bounds {
                Some((min_x, min_y, max_x, max_y)) => (
                    min_x.min(point.x),
                    min_y.min(point.y),
                    max_x.max(point.x),
                    max_y.max(point.y),
                ),
                None => (point.x, point.y, point.x, point.y),
            });
        }
    }
    bounds
}

pub(in crate::project_slice) fn model_center(
    traversal: &PreparedPostClassicTraversal,
) -> Option<(f64, f64)> {
    let (min_x, min_y, max_x, max_y) = model_bounds(traversal)?;
    Some(((min_x + max_x) * 0.5, (min_y + max_y) * 0.5))
}
