use crate::{
    geometry::{CoordinateScale, Point, Polyline},
    project_slice::{
        gcode_emit,
        island_print_order::{
            IslandPrintEntity, OrderedExtrusionLayer, PreparedPostIslandPrintOrder,
        },
        perimeters::classic::{
            gap_extrusion::GapFillEntity,
            materialize::{ExtrusionPath, Point3, Polyline3},
        },
    },
};

pub(in crate::project_slice) fn apply(prepared: &mut PreparedPostIslandPrintOrder) {
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let process = &traversal.resolved.views.full.process;
    if !process.gcode.enable_arc_fitting.0 || process.print.spiral_mode.0 {
        return;
    }
    let scale = traversal.scale;
    let tolerance = process.print.resolution.0;
    for layers in &mut prepared.objects {
        simplify_layers(layers, scale, tolerance);
    }
}

fn simplify_layers(layers: &mut [OrderedExtrusionLayer], scale: CoordinateScale, tolerance: f64) {
    for entity in layers
        .iter_mut()
        .flat_map(|layer| &mut layer.islands)
        .flat_map(|island| &mut island.entities)
    {
        match entity {
            IslandPrintEntity::Perimeter(collection) => {
                for path in collection
                    .entities
                    .iter_mut()
                    .flat_map(|entity| &mut entity.extrusion_loop.paths)
                {
                    simplify_path3(path, scale, tolerance);
                }
            }
            IslandPrintEntity::Fill(collection) => {
                for path in &mut collection.paths {
                    let mut points = path
                        .polyline
                        .points()
                        .iter()
                        .map(|point| (scale.unscale(point.x()), scale.unscale(point.y())))
                        .collect::<Vec<_>>();
                    gcode_emit::simplify_points(&mut points, tolerance);
                    path.polyline = Polyline::new(
                        points
                            .into_iter()
                            .map(|point| scaled_point(point, scale))
                            .collect(),
                    );
                }
            }
            IslandPrintEntity::Thin(entity) => match entity {
                GapFillEntity::Path(path) => simplify_path3(path, scale, tolerance),
                GapFillEntity::Loop(paths) => {
                    for path in paths {
                        simplify_path3(path, scale, tolerance);
                    }
                }
            },
        }
    }
}

fn scaled_point((x, y): (f64, f64), scale: CoordinateScale) -> Point {
    Point::new(
        (x / scale.factor()).round() as i64,
        (y / scale.factor()).round() as i64,
    )
}

fn simplify_path3(path: &mut ExtrusionPath, scale: CoordinateScale, tolerance: f64) {
    let z = path.polyline.points[0].z;
    let mut points = path
        .polyline
        .points
        .iter()
        .map(|point| (scale.unscale(point.x), scale.unscale(point.y)))
        .collect::<Vec<_>>();
    gcode_emit::simplify_points(&mut points, tolerance);
    path.polyline = Polyline3 {
        points: points
            .into_iter()
            .map(|(x, y)| Point3 {
                x: (x / scale.factor()).round() as i64,
                y: (y / scale.factor()).round() as i64,
                z,
            })
            .collect(),
    };
}
