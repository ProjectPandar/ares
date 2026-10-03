//! `TreeSupport3D.cpp::generate_initial_areas`'s per-overhang geometry.

pub(super) mod generation;
pub(super) mod raft_contact;

use std::borrow::Cow;

use super::super::model_volumes::{AvoidanceType, TreeModelVolumes};
use super::{
    TreeSettings,
    area_growth::{Expansion, safe_offset_inc},
};
use crate::geometry::{
    ClipperError, Coord, CoordinateScale, FillRule, JoinType, Polygon, difference_polygons_paths,
    difference_polygons_paths_with_safety_offset, intersection_polygons_paths,
    offset_expolygons_paths, union_ex, union_polygons_paths_pair,
};

pub(super) struct InitialAreas {
    pub(super) regular: Vec<Polygon>,
    pub(super) roofs: Vec<Polygon>,
}

pub(super) fn prepare_layer(
    raw_overhang: &[Polygon],
    layer_index: usize,
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    scale: CoordinateScale,
) -> Result<InitialAreas, ClipperError> {
    let mesh = &config.mesh;
    let minimum_xy = config.xy_distance > config.xy_min_distance;
    let forbidden_raw = if config.support_rests_on_model {
        volumes.collision(config.radius(0, 0.0), layer_index, minimum_xy)?
    } else {
        volumes.avoidance(
            (config.radius(0, 0.0), layer_index),
            AvoidanceType::Fast,
            false,
            minimum_xy,
        )?
    };
    let forbidden = offset_union(forbidden_raw, scale.scaled_delta(0.005) as f32)?;
    let mut regular = safe_offset_inc(
        raw_overhang,
        &forbidden,
        expansion(
            mesh.support_offset,
            (config.min_radius as f64 * 1.75 + config.xy_min_distance as f64) as Coord,
        ),
        scale,
    )?;
    let expanded_raw = if mesh.support_offset == 0 {
        Cow::Borrowed(raw_overhang)
    } else {
        Cow::Owned(offset_union(raw_overhang, mesh.support_offset as f32)?)
    };
    let mut remaining = intersection_polygons_paths(
        &difference_polygons_paths(
            &expanded_raw,
            &offset_union(&regular, (mesh.support_line_width as f64 * 0.5) as f32)?,
        )?,
        &forbidden,
    )?;
    let circle_length = if config.min_radius < mesh.support_line_width {
        config.min_radius / 2
    } else {
        let radius = scale.unscale(config.min_radius);
        let inset_radius = scale.unscale(config.min_radius - mesh.support_line_width / 2);
        scale.scaled_delta((radius * radius - inset_radius * inset_radius).sqrt()) as Coord
    };
    let extra_outset = (config.min_radius - mesh.support_line_width / 2).max(0)
        + if minimum_xy {
            mesh.support_line_width / 2
        } else {
            0
        };
    let mut extra_total = 0;
    while !remaining.is_empty() && extra_total + mesh.support_line_width / 8 < extra_outset {
        let step = if extra_total + 2 * mesh.support_line_width > config.min_radius {
            mesh.support_line_width / 8
        } else {
            circle_length
        }
        .min(extra_outset - extra_total);
        extra_total += step;
        let collision = volumes.collision(0, layer_index, true)?;
        let safe_step = config.xy_min_distance + mesh.support_line_width;
        remaining = difference_polygons_paths(
            &remaining,
            &safe_offset_inc(
                &regular,
                collision,
                expansion((1.5 * extra_total as f64) as Coord, safe_step),
                scale,
            )?,
        )?;
        regular = union_polygons_paths_pair(
            &regular,
            &difference_polygons_paths(
                &safe_offset_inc(
                    &remaining,
                    collision,
                    expansion(extra_total, safe_step),
                    scale,
                )?,
                &forbidden,
            )?,
        )?;
    }
    let mut roofs = if mesh.roof_layers > 0 {
        safe_offset_inc(
            raw_overhang,
            &forbidden,
            expansion(0, config.min_radius * 2 + config.xy_min_distance),
            scale,
        )?
    } else {
        Vec::new()
    };
    if mesh.roof_layers > 0 {
        // The native roof filter is gated by minimum_support_area, not minimum_roof_area.
        if mesh.minimum_support_area > 0 {
            roofs.retain(|polygon| polygon.area().abs() >= mesh.minimum_roof_area);
        }
        regular = difference_polygons_paths_with_safety_offset(&regular, &roofs)?;
    }
    if mesh.minimum_support_area > 0 {
        regular.retain(|polygon| polygon.area().abs() >= mesh.minimum_support_area as f64);
    }
    Ok(InitialAreas { regular, roofs })
}

fn expansion(distance: Coord, safe_step: Coord) -> Expansion {
    Expansion {
        distance,
        safe_step,
        unchecked_last: 0,
        minimum_offsets: 1,
    }
}

pub(super) fn offset_union(
    polygons: &[Polygon],
    distance: f32,
) -> Result<Vec<Polygon>, ClipperError> {
    offset_expolygons_paths(
        &union_ex(polygons, FillRule::NonZero)?,
        distance,
        JoinType::Miter,
        1.2,
    )
}
