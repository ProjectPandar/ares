//! `TreeSupport3D.cpp::increase_single_area/SupportElementInfluenceAreas`.

mod radius;

use super::super::model_volumes::TreeModelVolumes;
use super::{
    TreeSettings,
    area_growth::{Expansion, safe_offset_inc, safe_union},
    support_element::{AreaIncreaseSettings, SupportElement, SupportElementState},
};
use crate::geometry::{
    BoundingBox, ClipperError, Coord, CoordinateScale, Point, Polygon,
    clip_clipper_polygons_with_subject_bbox, difference_polygons_paths,
};

pub(super) struct IncreaseRequest<'a> {
    pub(super) settings: &'a AreaIncreaseSettings,
    pub(super) layer: usize,
    pub(super) parent: &'a SupportElement,
    pub(super) relevant_offset: &'a [Polygon],
    pub(super) overspeed: Coord,
    pub(super) merge_layer: bool,
}

#[derive(Default)]
pub(super) struct SupportElementInfluenceAreas {
    pub(super) influence_areas: Vec<Polygon>,
    pub(super) to_bp_areas: Vec<Polygon>,
    pub(super) to_model_areas: Vec<Polygon>,
}

impl SupportElementInfluenceAreas {
    fn selected(&self, state: &SupportElementState) -> &[Polygon] {
        if state.bits.to_buildplate {
            &self.to_bp_areas
        } else {
            &self.to_model_areas
        }
    }
}

pub(super) fn increase_single_area(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    request: &IncreaseRequest<'_>,
    areas: &mut SupportElementInfluenceAreas,
    scale: CoordinateScale,
) -> Result<Option<SupportElementState>, ClipperError> {
    let settings = request.settings;
    let mut current = request.parent.state.propagate_down();
    if settings.increase_radius {
        current.effective_radius_height += 1;
    }
    let radius = current.collision_radius(config);
    let tiny = scale.scaled_delta(0.001).powi(2);
    areas.influence_areas = if settings.move_area {
        let mut increased = request.relevant_offset.to_vec();
        if request.overspeed > 0 {
            let mut safe_step = if current.bits.use_min_xy_dist {
                config.xy_min_distance
            } else {
                config.xy_distance
            } + if config
                .z_distance_top_layers
                .min(config.z_distance_bottom_layers)
                > 0
            {
                config.mesh.min_feature_size
            } else {
                0
            };
            if safe_step == 0 {
                safe_step = scale.scaled_delta(0.1) as Coord;
            }
            increased = safe_offset_inc(
                &increased,
                volumes.wall_restriction(
                    request.parent.state.collision_radius(config),
                    request.layer,
                    request.parent.state.bits.use_min_xy_dist,
                )?,
                Expansion {
                    distance: request.overspeed,
                    safe_step,
                    unchecked_last: safe_step + radius,
                    minimum_offsets: 1,
                },
                scale,
            )?;
        }
        // The upstream const-input simplifier's returned geometry is discarded.
        increased
    } else {
        request.parent.influence_area.clone()
    };
    if request.merge_layer || current.bits.to_buildplate {
        areas.to_bp_areas = safe_union(
            &diff_clipped(
                &areas.influence_areas,
                volumes.avoidance(
                    (radius, request.layer - 1),
                    settings.kind,
                    false,
                    settings.use_min_distance,
                )?,
                scale,
            )?,
            &[],
            scale,
        )?;
        if !current.bits.to_buildplate && area(&areas.to_bp_areas) > tiny {
            current.bits.to_buildplate = true;
        }
    }
    if config.support_rests_on_model {
        if request.merge_layer || current.bits.to_model_gracious {
            areas.to_model_areas = safe_union(
                &diff_clipped(
                    &areas.influence_areas,
                    volumes.avoidance(
                        (radius, request.layer - 1),
                        settings.kind,
                        true,
                        settings.use_min_distance,
                    )?,
                    scale,
                )?,
                &[],
                scale,
            )?;
        }
        if !current.bits.to_model_gracious {
            if request.merge_layer && area(&areas.to_model_areas) >= tiny {
                current.bits.to_model_gracious = true;
            } else {
                areas.to_model_areas = safe_union(
                    &diff_clipped(
                        &areas.influence_areas,
                        volumes.collision(radius, request.layer - 1, settings.use_min_distance)?,
                        scale,
                    )?,
                    &[],
                    scale,
                )?;
            }
        }
    }
    if settings.increase_radius && area(areas.selected(&current)) > tiny {
        radius::grow(volumes, config, request, (&mut current, areas), scale)?;
    }
    Ok((area(areas.selected(&current)) > tiny).then_some(current))
}

fn area(polygons: &[Polygon]) -> f64 {
    polygons.iter().map(Polygon::area).sum()
}

pub(super) fn diff_clipped(
    subject: &[Polygon],
    clip: &[Polygon],
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    let bounds = BoundingBox::from_polygons(subject)
        .unwrap_or(BoundingBox::new(Point::new(0, 0), Point::new(0, 0)))
        .inflated(scale.scaled_delta(1e-4) as Coord);
    difference_polygons_paths(
        subject,
        &clip_clipper_polygons_with_subject_bbox(clip, bounds),
    )
}
