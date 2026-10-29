//! `TreeSupport3D.cpp::create_layer_pathing`.

use std::{collections::VecDeque, mem};

use super::super::model_volumes::TreeModelVolumes;
use super::{
    TreeSettings,
    area_growth::safe_union,
    layer_growth::{LayerGrowth, SupportElementMerging, increase_areas_one_layer},
    merge_areas::merge_influence_areas,
    single_area::SupportElementInfluenceAreas,
    support_element::SupportElement,
};
use crate::geometry::{ClipperError, CoordinateScale};

pub(super) fn create_layer_pathing(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    move_bounds: &mut [VecDeque<SupportElement>],
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let mut last_merge_layer = move_bounds.len();
    let mut new_element = false;
    let max_merge_every = (5000 / config.maximum_move_distance.max(100))
        .min(1000 / config.maximum_move_distance_slow.max(20))
        .min(3000 / config.mesh.layer_height) as usize;
    let mut merge_every = 1;
    for layer in (1..move_bounds.len()).rev() {
        let (below, above) = move_bounds.split_at_mut(layer);
        let previous = &mut above[0];
        if previous.is_empty() {
            continue;
        }
        let had_new_element = new_element;
        let merge_this_layer = had_new_element || last_merge_layer - layer >= merge_every;
        if had_new_element {
            merge_every = 1;
        }
        let mut influence_areas = previous
            .iter()
            .enumerate()
            .map(|(index, element)| {
                debug_assert!(!element.influence_area.is_empty());
                SupportElementMerging {
                    state: element.state,
                    parents: vec![index as i32],
                    areas: SupportElementInfluenceAreas::default(),
                    bbox: None,
                }
            })
            .collect::<Vec<_>>();
        increase_areas_one_layer(
            volumes,
            config,
            LayerGrowth {
                layer,
                merge_layer: merge_this_layer,
                merging: &mut influence_areas,
                parents: previous,
            },
            scale,
        )?;
        let current = &mut below[layer - 1];
        influence_areas.retain_mut(|element| {
            if element.areas.influence_areas.is_empty() {
                return false;
            }
            if !element.areas.to_bp_areas.is_empty() || !element.areas.to_model_areas.is_empty() {
                return true;
            }
            current.push_back(SupportElement {
                state: element.state,
                parents: mem::take(&mut element.parents),
                influence_area: mem::take(&mut element.areas.influence_areas),
            });
            false
        });
        new_element = !current.is_empty();
        if merge_this_layer {
            let count_before_merge = influence_areas.len();
            if count_before_merge > 1 {
                merge_influence_areas(volumes, config, layer, &mut influence_areas, scale)?;
            }
            let reduced_by_merging = count_before_merge > influence_areas.len();
            last_merge_layer = layer;
            if !reduced_by_merging && !had_new_element {
                merge_every = max_merge_every.min(merge_every + 1);
            }
        }
        for element in influence_areas {
            if element.areas.influence_areas.is_empty() {
                continue;
            }
            let influence_area = safe_union(&element.areas.influence_areas, &[], scale)?;
            current.push_back(SupportElement {
                state: element.state,
                parents: element.parents,
                influence_area,
            });
        }
    }
    Ok(())
}
