use super::*;

pub(super) struct LoopParameters {
    pub(super) loops_count: usize,
    pub(super) scaled_spacing: f32,
    pub(super) arc_tolerance: f64,
    pub(super) minimum_filament: f64,
    pub(super) e_per_path_mm: f64,
    pub(super) scale: CoordinateScale,
}

pub(super) fn make_loops(
    hull: &Polygon,
    distance: &mut f32,
    parameters: LoopParameters,
) -> Result<Vec<Vec<Point>>, SliceError> {
    let LoopParameters {
        loops_count,
        scaled_spacing,
        arc_tolerance,
        minimum_filament,
        e_per_path_mm,
        scale,
    } = parameters;
    let mut extruded_filament = 0.0;
    let mut group_loops = Vec::with_capacity(loops_count);
    while loops_count > 0
        && (group_loops.len() < loops_count || extruded_filament < minimum_filament)
    {
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
        extruded_filament += closed_length(&simplified) * scale.factor() * e_per_path_mm;
        group_loops.push(simplified);
    }
    Ok(group_loops)
}
