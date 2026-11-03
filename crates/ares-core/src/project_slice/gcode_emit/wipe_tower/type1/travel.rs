//! `GCode.cpp:594–710`: the Type1 approach's start and avoid box for the shared tower router.
//! The polyline points are `Point`s (grid ints) that `GCode.cpp:1034-1046`
//! feeds straight into `travel_to(p)` — never rescaled through `f32`.
use super::super::super::{motion, timelapse};
use super::super::postprocess;
use crate::geometry::CoordinateScale;
type Point = [i64; 2];

pub(super) fn path(
    source: &str,
    end: [f32; 2],
    state: &motion::EmitState,
    context: timelapse::Context<'_>,
    scale: CoordinateScale,
) -> Result<Vec<Point>, crate::SliceError> {
    let mut start = [0.0_f32; 2];
    // GCodeProcessor.cpp:3021: complete motion lines and first X/Y word.
    for line in source
        .split_inclusive('\n')
        .filter_map(|line| line.strip_suffix('\n'))
    {
        let line = line
            .trim_matches(' ')
            .trim_end_matches(';')
            .trim_end_matches(' ');
        if line.len() <= 5
            || !["G0 ", "G1 ", "G2 ", "G3 "]
                .iter()
                .any(|prefix| line.starts_with(prefix))
        {
            continue;
        }
        for (axis, key) in [" X", " Y"].iter().enumerate() {
            let Some((_, tail)) = line.split_once(key) else {
                continue;
            };
            let word = tail.split([' ', '\t', '\r', '\n', ';']).next().unwrap();
            if word.len() <= 8
                && let Ok(value) = word.parse::<f32>()
            {
                start[axis] = value;
            }
        }
    }
    let tower = state.wipe_tower.as_ref().unwrap();
    let bounds = tower.data.first_layer_bounds.unwrap();
    let translation = [
        tower.position[0] + tower.data.rib_offset[0],
        tower.position[1] + tower.data.rib_offset[1],
    ];
    // Tower and bed points take `plate_origin_2d` before `wipe_tower_point_to_object_point`
    // (`GCode.cpp:1015-1018, 1513-1522`).
    let plate_origin = state.xy_offset_f32();
    let local = |p: [f32; 2]| {
        let p = [p[0] + plate_origin[0], p[1] + plate_origin[1]];
        [
            ((f64::from(p[0]) - state.origin.0) / scale.factor()).round() as i64,
            ((f64::from(p[1]) - state.origin.1) / scale.factor()).round() as i64,
        ]
    };
    let min = bounds.min();
    let max = bounds.max();
    let corners = [
        [min.x(), min.y()],
        [max.x(), min.y()],
        [max.x(), max.y()],
        [min.x(), max.y()],
    ]
    .map(|p| {
        local(postprocess::transform(
            [scale.unscale(p[0]) as f32, scale.unscale(p[1]) as f32],
            translation,
            tower.angle,
        ))
    });
    let mut low = [i64::MAX; 2];
    let mut high = [i64::MIN; 2];
    for p in corners {
        for a in 0..2 {
            low[a] = low[a].min(p[a]);
            high[a] = high[a].max(p[a]);
        }
    }
    let end_local = local(end);
    // GCode.cpp:1513,8138: parsed G-code is in nozzle coordinates, unlike
    // tower points. It takes the plate origin, then gcode_to_point adds the
    // active nozzle offset and truncates.
    let start = [
        ((f64::from(start[0]) + f64::from(plate_origin[0]) - state.origin.0
            + state.extruder_offset.0)
            / scale.factor()) as i64,
        ((f64::from(start[1]) + f64::from(plate_origin[1]) - state.origin.1
            + state.extruder_offset.1)
            / scale.factor()) as i64,
    ];
    let bed = super::super::route::shared_printable_area(state, context, scale)?;
    Ok(super::super::route::path_to_wipe_tower(
        start,
        end_local,
        [low, high],
        &bed,
        scale,
    ))
}
