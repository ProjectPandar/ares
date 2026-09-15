use crate::{
    geometry::{CoordinateScale, Point, Polyline},
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

fn simplify_layers(
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
        FillExtrusionEntity::Path(path) => {
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
                path.fitting = gcode_emit::motion::simplify_points(
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
                path.fitting = Vec::new();
            }
            path.polyline = Polyline::new(
                points
                    .into_iter()
                    .map(|point| scaled_point(point, scale))
                    .collect(),
            );
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
    let z = path.polyline.points[0].z;
    let source_points = std::mem::take(&mut path.polyline.points);
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
    path.polyline = Polyline3 {
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
mod tests {
    use crate::ExtrusionRole;

    use super::fill_tolerance;

    #[test]
    fn task22o213_sparse_infill_uses_source_coarse_resolution() {
        assert_eq!(fill_tolerance(ExtrusionRole::InternalInfill, 0.0125), 0.04);
        assert_eq!(fill_tolerance(ExtrusionRole::SolidInfill, 0.0125), 0.0125);
    }
}
