//! The `WipeTower` and `WipeTower2` statics the tower estimate calls (`GCode/WipeTower.cpp:51,
//! 124-127, 1690-1824` and `GCode/WipeTower2.cpp:2151-2245` at v2.5.0-beta `345efec64`).

use std::collections::BTreeMap;
use std::f64::consts::{FRAC_PI_4, PI};

use super::EPSILON;
use super::config::{DynamicConfig, get_at};

/// `WipeTower::PurgeEstimate`: one filament's purge in the Type1 planner.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PurgeEstimate {
    pub(crate) prime_volume: f32,
    pub(crate) category: i32,
    pub(crate) filament_diameter: f32,
    pub(crate) filament_change_length: f32,
}

/// `WipeTower::min_depth_per_height`.
const MIN_DEPTH_PER_HEIGHT: [(f32, f32); 4] =
    [(5.0, 5.0), (100.0, 20.0), (250.0, 40.0), (350.0, 60.0)];

/// `nozzle_diameter_to_nozzle_change_width`.
const NOZZLE_CHANGE_WIDTH: [(f32, f32); 4] = [(0.2, 0.5), (0.4, 1.0), (0.6, 1.2), (0.8, 1.4)];

/// `wrapping_wipe_tower_depth`.
const WRAPPING_WIPE_TOWER_DEPTH: f64 = 10.0;

/// `Width_To_Nozzle_Ratio`.
const WIDTH_TO_NOZZLE_RATIO: f32 = 1.25;

/// `WipeTower::get_limit_depth_by_height`.
pub(crate) fn limit_depth_by_height(max_height: f32) -> f32 {
    for (index, &(height, depth)) in MIN_DEPTH_PER_HEIGHT.iter().enumerate() {
        if height >= max_height {
            return depth;
        }
        let Some(&(next_height, next_depth)) = MIN_DEPTH_PER_HEIGHT.get(index + 1) else {
            return depth;
        };
        if next_height > max_height {
            return depth + (max_height - height) / (next_height - height) * (next_depth - depth);
        }
    }
    unreachable!("the last entry returns")
}

/// `WipeTower::get_auto_brim_by_height`.
pub(crate) fn auto_brim_by_height(max_height: f32) -> f32 {
    if max_height < 100.0 {
        max_height / 100.0 * 8.0
    } else {
        8.0
    }
}

/// `WipeTower::estimate_brim_real_width`.
pub(crate) fn brim_real_width(
    brim_width: f32,
    nozzle_diameter: f32,
    first_layer_height: f32,
    type2: bool,
) -> f32 {
    if brim_width <= 0.0 {
        return brim_width;
    }
    let spacing =
        nozzle_diameter * WIDTH_TO_NOZZLE_RATIO - first_layer_height * (1.0 - FRAC_PI_4) as f32;
    if f64::from(spacing) <= EPSILON {
        return brim_width;
    }
    let loops = ((brim_width + spacing / 2.0) / spacing) as i32;
    loops as f32 * spacing + if type2 { 0.0 } else { spacing / 2.0 }
}

/// `WipeTower::get_wrapping_detection_depth`.
pub(crate) fn wrapping_detection_depth() -> f32 {
    WRAPPING_WIPE_TOWER_DEPTH as f32
}

/// `WipeTower::nozzle_change_perimeter_width`: an exact key match, as `std::map::find`.
fn nozzle_change_perimeter_width(nozzle_diameter: f32) -> f32 {
    NOZZLE_CHANGE_WIDTH
        .iter()
        .find(|&&(diameter, _)| diameter == nozzle_diameter)
        .map_or(
            2.0 * nozzle_diameter * WIDTH_TO_NOZZLE_RATIO,
            |&(_, width)| width,
        )
}

/// `align_ceil`.
fn align_ceil(value: f32, base: f32) -> f32 {
    (value / base).ceil() * base
}

/// `WipeTower::estimate_tower_blocks_depth`.
pub(crate) fn tower_blocks_depth(
    purges: &[PurgeEstimate],
    width: f32,
    (layer_height, nozzle_diameter, extra_spacing): (f32, f32, f32),
) -> f32 {
    if purges.is_empty()
        || f64::from(layer_height) < EPSILON
        || f64::from(nozzle_diameter) < EPSILON
    {
        return 0.0;
    }
    let pw = nozzle_diameter * WIDTH_TO_NOZZLE_RATIO;
    let ncpw = nozzle_change_perimeter_width(nozzle_diameter);
    let line_width = width - 2.0 * pw;
    if f64::from(line_width) <= EPSILON {
        return 0.0;
    }
    let line_area = |w: f32| layer_height * (w - layer_height * (1.0 - FRAC_PI_4) as f32);
    let extra_width = (extra_spacing - 1.0) * pw;
    let gap = pw + extra_width;
    let nc_gap = ncpw + extra_width;
    // Per category: (depth, smallest purge, filaments).
    let mut blocks = BTreeMap::<i32, (f32, f32, usize)>::new();
    for purge in purges {
        let block = blocks.entry(purge.category).or_default();
        let purge_depth = (purge.prime_volume / line_area(pw) / line_width).ceil() * gap;
        block.1 = if block.2 == 0 {
            purge_depth
        } else {
            block.1.min(purge_depth)
        };
        block.0 += purge_depth;
        block.2 += 1;
        if f64::from(purge.filament_change_length) > EPSILON {
            let filament_area = PI as f32 * purge.filament_diameter * purge.filament_diameter / 4.0;
            let nc_length = purge.filament_change_length * filament_area / line_area(ncpw);
            block.0 += (nc_length / (width - ncpw - pw)).ceil() * nc_gap;
        }
    }
    let mut depth = pw;
    for (block_depth, min_purge, filaments) in blocks.into_values() {
        depth += if filaments == purges.len() {
            block_depth - min_purge
        } else {
            block_depth
        };
    }
    depth
}

/// `WipeTower::rib_footprint_side`.
pub(crate) fn rib_footprint_side(
    (width, depth): (f32, f32),
    rib_width: f32,
    extra_rib_length: f32,
    max_height: f32,
) -> f32 {
    if f64::from(width) < EPSILON || f64::from(depth) < EPSILON {
        return 0.0;
    }
    let diagonal = (width * width + depth * depth).sqrt();
    let mut rib_length = diagonal;
    if f64::from(depth) + EPSILON < f64::from(limit_depth_by_height(max_height)) {
        rib_length = rib_length.max(limit_depth_by_height(max_height) * 2.0_f64.sqrt() as f32);
    }
    rib_length = diagonal.max(rib_length + extra_rib_length);
    let rib_w = rib_width.min(width.min(depth) / 2.0);
    let per_side = ((rib_length - diagonal) / 2.0 + rib_w / 2.0) / 2.0_f64.sqrt() as f32;
    width.max(depth) + 2.0 * per_side
}

/// `WipeTower::estimate_rib_tower_bbox_side`; `planner` is `(layer_height, nozzle_diameter,
/// extra_spacing)` and `rib` is `(rib_width, extra_rib_length)`.
pub(crate) fn rib_tower_bbox_side(
    purges: &[PurgeEstimate],
    width: f32,
    planner: (f32, f32, f32),
    rib: (f32, f32),
    max_height: f32,
) -> f32 {
    let (layer_height, nozzle_diameter, _) = planner;
    if purges.is_empty()
        || f64::from(width) < EPSILON
        || f64::from(layer_height) < EPSILON
        || f64::from(nozzle_diameter) < EPSILON
    {
        return 0.0;
    }
    let pw = nozzle_diameter * WIDTH_TO_NOZZLE_RATIO;
    let square = align_ceil(
        (tower_blocks_depth(purges, width, planner) * width).sqrt(),
        pw,
    );
    let depth = tower_blocks_depth(purges, square, planner);
    rib_footprint_side((square, depth), rib.0, rib.1, max_height)
}

/// `get_flush_volumes_matrix`: nozzle `nozzle_id`'s block of the flattened matrix.
fn flush_volumes_block(matrix: &[f64], nozzle_id: usize, nozzle_nums: usize) -> &[f64] {
    let block = matrix.len() / nozzle_nums;
    &matrix[nozzle_id * block..(nozzle_id + 1) * block]
}

/// `WipeTower2::extract_wipe_volumes`.
fn extract_wipe_volumes(config: &DynamicConfig) -> Vec<Vec<f32>> {
    let raw = config.option("flush_volumes_matrix");
    let raw = raw.floats();
    let nozzle_diameter = config.option("nozzle_diameter");
    let mut nozzle_nums = nozzle_diameter.floats().len().max(1);
    let mut extruders = (((raw.len() / nozzle_nums) as f64).sqrt() + EPSILON) as usize;
    if extruders * extruders * nozzle_nums != raw.len() {
        nozzle_nums = 1;
        extruders = ((raw.len() as f64).sqrt() + EPSILON) as usize;
    }
    let purge = config.option("purge_in_prime_tower").bool()
        && config.option("single_extruder_multi_material").bool();
    let mut volumes = vec![vec![0.0_f32; extruders]; extruders];
    if purge {
        let multiplier = config.option("flush_multiplier");
        for nozzle_id in 0..nozzle_nums {
            let block = flush_volumes_block(raw, nozzle_id, nozzle_nums);
            let scale = get_at(multiplier.floats(), nozzle_id);
            for (i, j) in (0..extruders).flat_map(|i| (0..extruders).map(move |j| (i, j))) {
                let volume = &mut volumes[i][j];
                *volume = volume.max((block[i * extruders + j] as f32 as f64 * scale) as f32);
            }
        }
    }
    let minimal = config.option("filament_minimal_purge_on_wipe_tower");
    for row in &mut volumes {
        for (j, volume) in row.iter_mut().enumerate() {
            *volume = volume.max(get_at(minimal.floats(), j) as f32);
        }
    }
    volumes
}

/// `WipeTower2::estimate_semm_flush_volume`.
pub(crate) fn semm_flush_volume(config: &DynamicConfig, filaments_cnt: usize) -> f32 {
    let volumes = extract_wipe_volumes(config);
    if volumes.is_empty() {
        return 0.0;
    }
    let mut maximum: f32 = volumes
        .iter()
        .map(|row| row.iter().copied().fold(f32::MIN, f32::max))
        .sum();
    maximum = maximum * filaments_cnt as f32 / volumes.len() as f32;
    // `maximum *= 0.6`: the product runs in double.
    (f64::from(maximum) * 0.6) as f32
}
