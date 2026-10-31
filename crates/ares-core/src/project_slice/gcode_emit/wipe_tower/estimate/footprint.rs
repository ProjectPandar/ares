//! `estimate_wipe_tower_footprint` (`GCode/WipeTowerEstimate.cpp:86-245` at v2.5.0-beta
//! `345efec64`).

use std::collections::BTreeSet;
use std::f64::consts::PI;

use super::config::DynamicConfig;
use super::statics::{
    PurgeEstimate, brim_real_width, limit_depth_by_height, rib_footprint_side, rib_tower_bbox_side,
    semm_flush_volume, tower_blocks_depth, wrapping_detection_depth,
};
use super::{
    EPSILON, Footprint, WipeTowerType, expand_mixed_filaments, float_at, has_any_mixed_filament,
    int_at, max_of, opt_float,
};

/// The Type1 planner's per-filament purges (`:150-173`).
fn type1_purges(
    config: &DynamicConfig,
    filament_ids: &[u32],
    prime_volume: f64,
    dual_nozzle: bool,
) -> Vec<PurgeEstimate> {
    let saving_mode = config.option("prime_volume_mode").key() == "Saving";
    let mut nozzles = BTreeSet::new();
    let mut longest_ramming = 0;
    let change_length = |id: u32| float_at(config, "filament_change_length", id, 0.0);
    let mut purges = Vec::with_capacity(filament_ids.len());
    for (i, &id) in filament_ids.iter().enumerate() {
        purges.push(PurgeEstimate {
            prime_volume: if saving_mode {
                15.0
            } else {
                float_at(config, "filament_prime_volume", id, prime_volume) as f32
            },
            category: int_at(config, "filament_adhesiveness_category", id, 0) as i32,
            filament_diameter: float_at(config, "filament_diameter", id, 1.75) as f32,
            filament_change_length: 0.0,
        });
        if dual_nozzle {
            nozzles.insert(int_at(config, "filament_map", id, 1));
            if change_length(id) > change_length(filament_ids[longest_ramming]) {
                longest_ramming = i;
            }
        }
    }
    if nozzles.len() > 1 {
        purges[longest_ramming].filament_change_length =
            (change_length(filament_ids[longest_ramming]) * (nozzles.len() - 1) as f64) as f32;
    }
    purges
}

/// The tower's footprint for the filaments `filament_slots` (0-based), sliced at `layer_height`
/// up to `max_object_height`.
pub(crate) fn estimate_footprint(
    config: &DynamicConfig,
    tower_type: WipeTowerType,
    filament_slots: &[u32],
    mut layer_height: f64,
    max_object_height: f64,
) -> Footprint {
    let mut footprint = Footprint {
        height: max_object_height,
        ..Footprint::default()
    };
    let is_mixed = config.option("filament_is_mixed");
    let filament_ids = if has_any_mixed_filament(is_mixed.bools()) {
        expand_mixed_filaments(
            filament_slots,
            is_mixed.bools(),
            config.option("filament_mixed_components").strings(),
        )
    } else {
        filament_slots.to_vec()
    };
    let filaments_cnt = filament_ids.len();
    if filaments_cnt == 0 || layer_height < EPSILON {
        return footprint;
    }
    let first_layer_height = opt_float(config, "initial_layer_print_height");
    if first_layer_height > EPSILON {
        layer_height = layer_height.min(first_layer_height);
    }
    let type1 = tower_type == WipeTowerType::Type1;
    let width = opt_float(config, "prime_tower_width");
    let prime_volume = opt_float(config, "prime_volume");
    let extra_spacing = opt_float(
        config,
        if type1 {
            "prime_tower_infill_gap"
        } else {
            "wipe_tower_extra_spacing"
        },
    ) / 100.0;
    let rib_width = opt_float(config, "wipe_tower_rib_width");
    let extra_rib_length = opt_float(config, "wipe_tower_extra_rib_length");
    let nozzles = config.option("nozzle_diameter");
    let nozzle_diameter = nozzles.floats().first().copied().unwrap_or(0.4);
    let dual_nozzle = nozzles.floats().len() == 2;
    let wall_type = config.option("wipe_tower_wall_type");
    let rib_wall = wall_type.key() == "rib";
    let smooth_timelapse = config.option("timelapse_type").key() == "1";
    let wrapping = config.option("enable_wrapping_detection").bool();
    let need_wipe_tower = smooth_timelapse || wrapping;
    if filaments_cnt < 2 && !need_wipe_tower {
        return footprint;
    }
    let purge_count = match filaments_cnt {
        0 | 1 => 0,
        _ if dual_nozzle => filaments_cnt,
        _ => filaments_cnt - 1,
    };
    let mut volume = prime_volume * purge_count as f64;
    if dual_nozzle {
        let length = max_of(config, "filament_change_length", 0.0);
        let diameter = max_of(config, "filament_diameter", 1.75);
        volume += length * PI * diameter * diameter / 4.0 * (filaments_cnt / 2) as f64;
    }
    let semm_flush = config.option("purge_in_prime_tower").bool()
        && config.option("single_extruder_multi_material").bool();
    if semm_flush {
        volume = f64::from(semm_flush_volume(config, filaments_cnt));
    }
    let purges = if type1 && filaments_cnt > 1 {
        type1_purges(config, &filament_ids, prime_volume, dual_nozzle)
    } else {
        Vec::new()
    };
    let min_depth = f64::from(limit_depth_by_height(max_object_height as f32));
    let perimeter_width = nozzle_diameter as f32 * 1.25;
    let idle_depth = if type1 && wrapping && !smooth_timelapse {
        f64::from(wrapping_detection_depth())
    } else {
        min_depth
    };
    let planner = (
        layer_height as f32,
        nozzle_diameter as f32,
        extra_spacing as f32,
    );
    if rib_wall {
        let rib = (rib_width as f32, extra_rib_length as f32);
        let side = if !purges.is_empty() {
            rib_tower_bbox_side(
                &purges,
                width as f32,
                planner,
                rib,
                max_object_height as f32,
            )
        } else {
            let has_purge = !type1 && volume > EPSILON;
            let square = if has_purge {
                (volume / layer_height * extra_spacing).sqrt()
            } else {
                idle_depth
            };
            rib_footprint_side(
                (square as f32, square as f32),
                rib.0,
                rib.1,
                max_object_height as f32,
            )
        };
        footprint.width = f64::from(side);
        footprint.depth = f64::from(side);
    } else {
        let depth = if type1 {
            if purges.is_empty() {
                idle_depth
            } else {
                (min_depth + f64::from(perimeter_width)).max(f64::from(tower_blocks_depth(
                    &purges,
                    width as f32,
                    planner,
                )))
            }
        } else {
            let mut depth = volume / (layer_height * width);
            if !semm_flush {
                depth *= extra_spacing;
            }
            min_depth.max(depth)
        };
        footprint.width = width;
        footprint.depth = depth;
    }
    footprint.brim_width = opt_float(config, "prime_tower_brim_width");
    if footprint.brim_width < 0.0 {
        footprint.brim_width = f64::from(super::statics::auto_brim_by_height(
            max_object_height as f32,
        ));
    }
    let brim_layer = if first_layer_height > EPSILON {
        first_layer_height
    } else {
        layer_height
    };
    footprint.brim_width = f64::from(brim_real_width(
        footprint.brim_width as f32,
        nozzle_diameter as f32,
        brim_layer as f32,
        !type1,
    ));
    footprint
}
