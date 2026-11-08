//! `TreeSupport3D.cpp::sample_overhang_area`.

mod roof_stack;

use super::{
    infill_lines::{self, InfillLayer},
    initial_areas::offset_union,
    rich_interface_placer::{RichInterfacePlacer, TipLayers},
    tip_lines,
    tip_sampling::ensure_distance,
};
use crate::geometry::{
    ClipperError, Coord, CoordinateScale, JoinType, Line, Polygon, Polyline,
    difference_polygons_ex, offset_expolygons_paths,
};

pub(super) struct OverhangSample {
    pub(super) polygons: Vec<Polygon>,
    pub(super) large_horizontal_roof: bool,
    pub(super) layer_index: usize,
    pub(super) roof_layers: usize,
    pub(super) connect_length: Coord,
}

pub(super) fn sample(
    mut input: OverhangSample,
    placer: &mut RichInterfacePlacer<'_, '_, '_>,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let mut lines = Vec::new();
    let mut roof_depth = 0;
    let mut generation_depth = 0;
    if input.large_horizontal_roof {
        (lines, roof_depth, generation_depth) = roof_stack::generate(&mut input, placer, scale)?;
    }
    if lines.is_empty() {
        let config = placer.interface.config;
        let supports_roof = roof_depth > 0;
        let continuous_tips = !supports_roof && input.large_horizontal_roof;
        let mut polylines = ensure_distance(
            infill_lines::generate(
                &input.polygons,
                placer.interface.parameters,
                InfillLayer {
                    roof: supports_roof,
                    layer_index: input.layer_index - generation_depth,
                    support_infill_distance: if supports_roof {
                        config.mesh.roof_line_distance
                    } else {
                        config.mesh.branch_distance
                    },
                },
                scale,
            )?,
            if continuous_tips {
                (config.min_radius / 2) as f64
            } else {
                input.connect_length as f64
            },
            1,
            scale,
        );
        let point_count: usize = polylines.iter().map(|line| line.points().len()).sum();
        let total_length: f64 = input
            .polygons
            .iter()
            .map(|polygon| {
                let points = polygon.points();
                if points.len() <= 1 {
                    return 0.0;
                }
                let closing = Line::new(points[points.len() - 1], points[0]).length();
                points.windows(2).fold(closing, |length, points| {
                    length + Line::new(points[0], points[1]).length()
                })
            })
            .sum();
        let minimum_points =
            ((total_length / input.connect_length as f64) as Coord).clamp(1, 3) as usize;
        if point_count <= minimum_points {
            let reduced = offset_union(
                &input.polygons,
                (-config.mesh.support_line_width as f64 / 2.2) as f32,
            )?;
            let use_reduced = !reduced.is_empty()
                && offset_expolygons_paths(
                    &difference_polygons_ex(&input.polygons, &reduced)?,
                    config.mesh.support_line_width.max(input.connect_length) as f32,
                    JoinType::Miter,
                    1.2,
                )?
                .iter()
                .map(Polygon::area)
                .sum::<f64>()
                    < scale.scaled_delta(0.001).powi(2);
            let outlines = if use_reduced {
                &reduced
            } else {
                &input.polygons
            };
            polylines = ensure_distance(
                outlines.iter().map(Polygon::split_at_first_point).collect(),
                input.connect_length as f64,
                minimum_points,
                scale,
            );
        }
        lines = tip_lines::convert(
            &polylines,
            input.layer_index - roof_depth,
            placer.volumes,
            config,
        )?;
    }
    if roof_depth >= input.layer_index && input.large_horizontal_roof {
        placer
            .interface
            .add_roof_build_plate(input.polygons, roof_depth);
    } else {
        placer.add_points_along_lines(
            lines,
            TipLayers {
                insert_layer: input.layer_index - roof_depth,
                roof_tip_layers: if placer.force_tip_to_roof {
                    input.roof_layers - roof_depth
                } else {
                    0
                },
                supports_roof_layers: roof_depth,
                dont_move_until: if input.roof_layers > 0 {
                    input.roof_layers - roof_depth
                } else {
                    0
                },
            },
        )?;
    }
    Ok(())
}

fn roof_lines(
    area: &[Polygon],
    placer: &RichInterfacePlacer<'_, '_, '_>,
    layer_index: usize,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    infill_lines::generate(
        area,
        placer.interface.parameters,
        InfillLayer {
            roof: true,
            layer_index,
            support_infill_distance: placer.interface.config.mesh.roof_line_distance,
        },
        scale,
    )
}
