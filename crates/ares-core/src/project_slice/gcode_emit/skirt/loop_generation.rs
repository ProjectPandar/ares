//! Print.cpp:2728-2778: a fresh filament budget for each group hull.

use super::*;

pub(super) struct LoopParameters<'a> {
    pub(super) loops_count: usize,
    pub(super) scaled_spacing: f32,
    pub(super) arc_tolerance: f64,
    pub(super) minimum_filament: f64,
    pub(super) e_per_path_mm: &'a [f64],
    pub(super) scale: CoordinateScale,
}

pub(super) fn make_loops(
    hull: &Polygon,
    distance: &mut f32,
    parameters: LoopParameters<'_>,
) -> Result<Vec<Vec<Point>>, SliceError> {
    let LoopParameters {
        loops_count,
        scaled_spacing,
        arc_tolerance,
        minimum_filament,
        e_per_path_mm,
        scale,
    } = parameters;
    let mut extruded_filament = vec![0.0; e_per_path_mm.len()];
    let mut extruder = 0;
    let mut remaining = loops_count;
    let mut group_loops = Vec::with_capacity(loops_count);
    while remaining > 0 {
        *distance += scaled_spacing;
        let offset = offset_paths(
            std::slice::from_ref(hull),
            distance.abs(),
            JoinType::Round,
            arc_tolerance,
        )
        .map_err(|_| SliceError::InvalidInput("skirt offset failed".into()))?;
        let Some(loop_polygon) = offset.first() else {
            break;
        };
        let simplified = simplify_closed_points(
            loop_polygon.points().to_vec(),
            checked_scale(scale, 0.05)? as f64,
        );
        if simplified.len() < 3 {
            break;
        }
        if minimum_filament > 0.0 {
            extruded_filament[extruder] +=
                closed_length(&simplified) * scale.factor() * e_per_path_mm[extruder];
            let below_minimum = extruded_filament[extruder] < minimum_filament;
            remaining += usize::from(below_minimum && remaining == 1);
            if !below_minimum && extruder + 1 < e_per_path_mm.len() {
                extruder += 1;
            }
        }
        group_loops.push(simplified);
        remaining -= 1;
    }
    Ok(group_loops)
}
