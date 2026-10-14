use crate::{
    geometry::{CoordinateScale, Point},
    project_slice::{
        fill_entities::FillExtrusionEntity,
        gcode_emit,
        island_print_order::{
            IslandPrintEntity, OrderedExtrusionLayer, PreparedPostIslandPrintOrder,
        },
        perimeters::classic::{
            entity_collections::ExtrusionEntity,
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
    // `LayerRegion::simplify_path` (`LayerRegion.cpp:1071-1100`): with arc
    // fitting enabled (and not in spiral mode) each path is arc-fitted
    // then DP-simplified; otherwise every path still gets a plain DP
    // simplify at the printer resolution — the phase never skips.
    let arc_fitting = process.gcode.enable_arc_fitting.0 && !process.print.spiral_mode.0;
    let scale = traversal.scale;
    let tolerance = process.print.resolution.0;
    for layers in &mut prepared.objects {
        simplify_layers(layers, scale, tolerance, arc_fitting);
    }
}

pub(in crate::project_slice) fn simplify_support_layers(
    layers: &mut [Vec<super::support_material::InstalledSupportLayer>],
    scale: CoordinateScale,
    tolerance: f64,
    arc_fitting: bool,
) {
    let support_tolerance = if arc_fitting { 0.0375 } else { tolerance };
    for entity in layers
        .iter_mut()
        .flat_map(|layers| layers.iter_mut())
        .flat_map(|layer| &mut layer.support_fills)
    {
        entity.for_each_path_mut(&mut |path| {
            simplify_polyline3(&mut path.polyline, scale, support_tolerance, arc_fitting);
        });
    }
}

pub(in crate::project_slice) fn simplify_layers(
    layers: &mut [OrderedExtrusionLayer],
    scale: CoordinateScale,
    tolerance: f64,
    arc_fitting: bool,
) {
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
                    // `LayerRegion::simplify_multi_path` simplifies each
                    // sub-path exactly like a loop sub-path
                    // (`LayerRegion.cpp:1089-1125`).
                    .flat_map(|entity| match entity {
                        ExtrusionEntity::Loop(ordered) => &mut ordered.extrusion_loop.paths,
                        ExtrusionEntity::MultiPath(multi_path) => &mut multi_path.paths,
                        ExtrusionEntity::Path(path) => std::slice::from_mut(path),
                    })
                {
                    simplify_path3(path, scale, tolerance, arc_fitting);
                }
            }
            IslandPrintEntity::Fill(entity) => {
                simplify_fill_entity(entity, scale, tolerance, arc_fitting);
            }
            IslandPrintEntity::FillCollection(collection) => {
                for entity in &mut collection.entities {
                    simplify_fill_entity(entity, scale, tolerance, arc_fitting);
                }
            }
            IslandPrintEntity::Thin(entity) => {
                simplify_gap_entity(entity, scale, tolerance, arc_fitting)
            }
        }
    }
}

fn simplify_fill_entity(
    entity: &mut FillExtrusionEntity,
    scale: CoordinateScale,
    tolerance: f64,
    arc_fitting: bool,
) {
    match entity {
        FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => {
            // ExtrusionEntity.cpp:33-47: contour samples are already simplified.
            if path.z_contoured {
                return;
            }
            let mut points = path
                .polyline
                .points()
                .iter()
                .map(|point| (scale.unscale(point.x()), scale.unscale(point.y())))
                .collect::<Vec<_>>();
            // The coarse sparse-infill tolerance applies only in the
            // arc-fitting branch (`LayerRegion.cpp:1081`).
            let tolerance = if arc_fitting {
                fill_tolerance(path.role, tolerance)
            } else {
                tolerance
            };
            if arc_fitting {
                path.polyline.fitting = gcode_emit::motion::simplify_points(
                    &mut points,
                    tolerance,
                    units_per_mm(scale),
                );
            } else {
                gcode_emit::motion::simplify_linear_points(
                    &mut points,
                    tolerance,
                    units_per_mm(scale),
                );
                path.polyline.fitting = Vec::new();
            }
            path.polyline.points = points
                .into_iter()
                .map(|point| {
                    let point = scaled_point(point, scale);
                    Point3 {
                        x: point.x(),
                        y: point.y(),
                        z: 0,
                    }
                })
                .collect();
        }
        FillExtrusionEntity::VariableWidth(entity) => {
            simplify_gap_entity(entity, scale, tolerance, arc_fitting);
        }
    }
}

fn simplify_gap_entity(
    entity: &mut GapFillEntity,
    scale: CoordinateScale,
    tolerance: f64,
    arc_fitting: bool,
) {
    match entity {
        GapFillEntity::Path(path) => simplify_path3(path, scale, tolerance, arc_fitting),
        GapFillEntity::Loop(paths) => {
            for path in paths {
                simplify_path3(path, scale, tolerance, arc_fitting);
            }
        }
    }
}

fn fill_tolerance(role: crate::ExtrusionRole, configured: f64) -> f64 {
    if role == crate::ExtrusionRole::InternalInfill {
        0.04
    } else {
        configured
    }
}

fn units_per_mm(scale: CoordinateScale) -> f64 {
    scale.factor().recip()
}

fn scaled_point((x, y): (f64, f64), scale: CoordinateScale) -> Point {
    Point::new(
        (x / scale.factor()).round() as i64,
        (y / scale.factor()).round() as i64,
    )
}

fn simplify_path3(
    path: &mut ExtrusionPath,
    scale: CoordinateScale,
    tolerance: f64,
    arc_fitting: bool,
) {
    // ExtrusionEntity.cpp:33-47: ContourZ already simplifies its own 3D samples.
    if path.z_contoured {
        return;
    }
    simplify_polyline3(&mut path.polyline, scale, tolerance, arc_fitting);
}

fn simplify_polyline3(
    polyline: &mut Polyline3,
    scale: CoordinateScale,
    tolerance: f64,
    arc_fitting: bool,
) {
    let z = polyline.points[0].z;
    let source_points = std::mem::take(&mut polyline.points);
    let mut points = source_points
        .iter()
        .map(|point| (scale.unscale(point.x), scale.unscale(point.y)))
        .collect::<Vec<_>>();
    let fitting = if arc_fitting {
        gcode_emit::motion::simplify_points(&mut points, tolerance, units_per_mm(scale))
    } else {
        gcode_emit::motion::simplify_linear_points(&mut points, tolerance, units_per_mm(scale));
        Vec::new()
    };
    *polyline = Polyline3 {
        points: points
            .into_iter()
            .map(|(x, y)| Point3 {
                x: (x / scale.factor()).round() as i64,
                y: (y / scale.factor()).round() as i64,
                z,
            })
            .collect(),
        fitting,
    };
}
#[cfg(test)]
mod tests;
