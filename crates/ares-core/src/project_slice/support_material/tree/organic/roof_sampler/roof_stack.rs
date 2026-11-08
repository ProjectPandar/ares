//! Continuous roof descent within `sample_overhang_area`.

use super::{
    OverhangSample, RichInterfacePlacer, ensure_distance, offset_union, roof_lines, tip_lines,
};
use crate::geometry::{ClipperError, CoordinateScale, Polygon, difference_polygons_paths};
use crate::project_slice::support_material::tree::model_volumes::AvoidanceType;

pub(super) fn generate(
    input: &mut OverhangSample,
    placer: &mut RichInterfacePlacer<'_, '_, '_>,
    scale: CoordinateScale,
) -> Result<(tip_lines::LineInformations, usize, usize), ClipperError> {
    let config = placer.interface.config;
    let mut added_roofs = vec![Vec::new(); input.roof_layers];
    let mut last_overhang = input.polygons.clone();
    let mut lines = Vec::new();
    let mut depth = 0;
    while depth < input.roof_layers && input.layer_index - depth >= 1 {
        let minimum_xy = config.xy_distance > config.xy_min_distance;
        let radius = config.radius(0, 0.0);
        let layer = input.layer_index - depth - 1;
        let forbidden_raw = if config.support_rests_on_model {
            placer.volumes.collision(radius, layer, minimum_xy)?
        } else {
            placer
                .volumes
                .avoidance((radius, layer), AvoidanceType::Fast, false, minimum_xy)?
        };
        let forbidden = offset_union(forbidden_raw, scale.scaled_delta(0.005) as f32)?;
        let next_area = difference_polygons_paths(&input.polygons, &forbidden)?;
        if next_area.iter().map(Polygon::area).sum::<f64>() < config.mesh.minimum_roof_area {
            if depth > 0 {
                lines = previous_roof_lines(
                    (
                        &last_overhang,
                        input.layer_index - (depth - 1),
                        input.connect_length,
                    ),
                    placer,
                    scale,
                )?;
            }
            break;
        }
        added_roofs[depth] = input.polygons.clone();
        last_overhang = std::mem::replace(&mut input.polygons, next_area);
        depth += 1;
    }
    let mut generation_depth = depth.max(1) - 1;
    if lines.is_empty()
        && depth != 0
        && roof_lines(
            &input.polygons,
            placer,
            input.layer_index - generation_depth,
            scale,
        )?
        .is_empty()
    {
        for (index, roof) in added_roofs.iter().enumerate().take(depth) {
            if roof_lines(roof, placer, input.layer_index - index, scale)?.is_empty() {
                depth = index;
                generation_depth = depth.max(1) - 1;
                break;
            }
        }
    }
    added_roofs.truncate(depth);
    placer.interface.add_roofs(added_roofs, input.layer_index);
    Ok((lines, depth, generation_depth))
}

fn previous_roof_lines(
    (area, layer, connect_length): (&[Polygon], usize, crate::geometry::Coord),
    placer: &mut RichInterfacePlacer<'_, '_, '_>,
    scale: CoordinateScale,
) -> Result<tip_lines::LineInformations, ClipperError> {
    let polylines = ensure_distance(
        roof_lines(area, placer, layer, scale)?,
        connect_length as f64,
        1,
        scale,
    );
    let config = placer.interface.config;
    let converted = tip_lines::convert(&polylines, layer, placer.volumes, config)?;
    Ok(tip_lines::split(&converted, |point| {
        tip_lines::valid_on_next_layer(point, layer, placer.volumes, config)
    })?
    .0)
}
