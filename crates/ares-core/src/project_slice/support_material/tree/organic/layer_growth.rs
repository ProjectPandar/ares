//! `TreeSupport3D.cpp::increase_areas_one_layer/SupportElementMerging`.

mod movement;
mod settings_order;

use std::collections::VecDeque;

use super::super::model_volumes::{AvoidanceType, TreeModelVolumes};
use super::{
    TreeSettings,
    area_growth::safe_union,
    single_area::{
        IncreaseRequest, SupportElementInfluenceAreas, diff_clipped, increase_single_area,
    },
    support_element::{SupportElement, SupportElementState},
};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, JoinType, Polygon, offset_open_paths,
    union_polygons_paths_pair,
};
use movement::Movement;

pub(super) struct SupportElementMerging {
    pub(super) state: SupportElementState,
    pub(super) parents: Vec<i32>,
    pub(super) areas: SupportElementInfluenceAreas,
    pub(super) bbox: Option<BoundingBox>,
}

pub(super) struct LayerGrowth<'a> {
    pub(super) layer: usize,
    pub(super) merge_layer: bool,
    pub(super) merging: &'a mut [SupportElementMerging],
    pub(super) parents: &'a mut VecDeque<SupportElement>,
}

pub(super) fn increase_areas_one_layer(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    growth: LayerGrowth<'_>,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    for element in growth.merging {
        debug_assert_eq!(element.parents.len(), 1);
        let parent = &mut growth.parents[element.parents[0] as usize];
        increase_one(
            volumes,
            config,
            (growth.layer, growth.merge_layer),
            (element, parent),
            scale,
        )?;
    }
    Ok(())
}

fn increase_one(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    location: (usize, bool),
    elements: (&mut SupportElementMerging, &mut SupportElement),
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let (layer, merge_layer) = location;
    let (element, parent) = elements;
    let propagated = parent.state.propagate_down();
    // Native primes the wall cache even if no candidate subsequently moves.
    volumes.wall_restriction(
        parent.state.collision_radius(config),
        layer,
        parent.state.bits.use_min_xy_dist,
    )?;
    let mut movement = Movement::new(volumes, config, parent, layer, scale);
    let order = settings_order::ordered(
        volumes,
        config,
        (&propagated, parent),
        (&movement, merge_layer),
        layer,
    )?;
    let mut areas = SupportElementInfluenceAreas::default();
    for settings in order {
        if settings.move_area {
            movement.offset(
                &parent.influence_area,
                volumes.wall_restriction(
                    parent.state.collision_radius(config),
                    layer,
                    parent.state.bits.use_min_xy_dist,
                )?,
                settings.increase_speed,
                scale,
            )?;
        }
        areas.influence_areas.clear();
        let error_area = if settings.no_error {
            Vec::new()
        } else {
            let lines = parent
                .influence_area
                .iter()
                .map(|polygon| Polygon::new(polygon.split_at_first_point().into_points()))
                .collect::<Vec<_>>();
            let lines_offset = offset_open_paths(
                &lines,
                scale.scaled_delta(0.005) as f32,
                JoinType::Miter,
                1.2,
            )?;
            union_polygons_paths_pair(&parent.influence_area, &lines_offset)?
        };
        let request = IncreaseRequest {
            settings: &settings,
            layer,
            parent,
            relevant_offset: if settings.no_error {
                movement.selected(settings.increase_speed)
            } else {
                &error_area
            },
            overspeed: if settings.no_error {
                0
            } else {
                (movement.fast_speed as f64 * 1.5) as _
            },
            merge_layer,
        };
        let Some(mut state) = increase_single_area(volumes, config, &request, &mut areas, scale)?
        else {
            continue;
        };
        if !settings.no_error {
            state.bits.lost = true;
        }
        state.last_area_increase = settings;
        let bypass = !settings.move_area
            || (settings.use_min_distance && (state.distance_to_top as usize) < config.tip_layers);
        if settings.move_area {
            state.dont_move_until = 0;
        } else {
            state.result_on_layer = parent.state.result_on_layer;
        }
        state.bits.can_use_safe_radius = settings.kind != AvoidanceType::Fast;
        if !settings.use_min_distance {
            state.bits.use_min_xy_dist = false;
        }
        let collision = volumes.collision(
            state.collision_radius(config),
            layer - 1,
            state.bits.use_min_xy_dist,
        )?;
        let difference = diff_clipped(&areas.influence_areas, collision, scale)?;
        let routed = safe_union(&areas.to_bp_areas, &areas.to_model_areas, scale)?;
        let influence = safe_union(&difference, &routed, scale)?;
        element.state = state;
        element.bbox = Some(
            BoundingBox::from_polygons(&influence)
                .expect("successful native growth has a nonempty influence area")
                .inflated(scale.scaled_delta(1e-4) as _),
        );
        element.areas.influence_areas = influence;
        if !bypass {
            if state.bits.to_buildplate {
                element.areas.to_bp_areas = areas.to_bp_areas;
            }
            if config.support_rests_on_model {
                element.areas.to_model_areas = areas.to_model_areas;
            }
        }
        return Ok(());
    }
    parent.state.result_on_layer = None;
    parent.state.bits.to_model_gracious = false;
    parent.state.bits.very_lost = true;
    Ok(())
}
