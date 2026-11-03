//! `WipeTowerIntegration::travel_to_tower_gap` (`GCode.cpp:1029-1069`, v2.5.0-beta `345efec64`):
//! the approach waypoints that enter the gap-wall tower through its wall opening.
use super::super::{motion, timelapse};
use super::{generator::use_gap_wall, route};
use crate::{
    ProcessWipeTowerWallType, SliceError,
    geometry::{CoordinateScale, Point as GridPoint, Polygon},
};
type Point = [i64; 2];

/// The route's points before `start_wipe_pos`; none when the gap wall is off, the approach starts
/// over the tower body, or the clamped avoid box is empty.
pub(super) fn waypoints(
    state: &motion::EmitState,
    context: timelapse::Context<'_>,
    [route_start, start_wipe_pos]: [Point; 2],
    transform_wt2_pt: impl Fn([f32; 2]) -> [f32; 2],
    scale: CoordinateScale,
) -> Result<Vec<Point>, SliceError> {
    let print = &context.traversal.resolved.views.full.process.print;
    if !use_gap_wall(print) {
        return Ok(Vec::new());
    }
    let data = &state
        .wipe_tower
        .as_ref()
        .expect("active tower integration")
        .data;
    // `scaled(bbx).polygon()` through `transform_wt2_pt` and `wipe_tower_point_to_object_point`.
    let tower_polygon = |[min, max]: [[f64; 2]; 2]| {
        let scaled = |v: f64| (v / scale.factor()) as i64;
        let (min, max) = (
            [scaled(min[0]), scaled(min[1])],
            [scaled(max[0]), scaled(max[1])],
        );
        [min, [max[0], min[1]], max, [min[0], max[1]]].map(|[x, y]| {
            motion::tower::object_point_grid(
                transform_wt2_pt([scale.unscale(x) as f32, scale.unscale(y) as f32]),
                state,
                scale,
            )
        })
    };
    let body_width = if print.wipe_tower_wall_type == ProcessWipeTowerWallType::Rib {
        data.depth
    } else {
        print.prime_tower_width.0 as f32
    };
    let body = tower_polygon([[0.0; 2], [f64::from(body_width), f64::from(data.depth)]]);
    let body = Polygon::new(body.map(|[x, y]| GridPoint::new(x, y)).to_vec());
    if route::contains(&[body], route_start, true) {
        return Ok(Vec::new());
    }
    let bed = route::shared_printable_area(state, context, scale)?;
    // `WipeTower2::get_bbx` (`WipeTower2.hpp:91-97`).
    let bbx = match data.first_layer_bounds {
        Some(bounds) => [
            [
                scale.unscale(bounds.min().x()),
                scale.unscale(bounds.min().y()),
            ],
            [
                scale.unscale(bounds.max().x()),
                scale.unscale(bounds.max().y()),
            ],
        ],
        None => {
            let brim = f64::from(data.brim_width);
            [
                [-brim, -brim],
                [f64::from(data.width) + brim, f64::from(data.depth) + brim],
            ]
        }
    };
    let corners = tower_polygon(bbx);
    let (mut low, mut high) = ([i64::MAX; 2], [i64::MIN; 2]);
    for p in corners {
        for a in 0..2 {
            (low[a], high[a]) = (low[a].min(p[a]), high[a].max(p[a]));
        }
    }
    // The bed extents shrunk by `scaled(wipe_tower_routing_clearance) + SCALED_EPSILON`.
    let shrink = ((2.0 / scale.factor()) as i64 as f64 + 1e-4 / scale.factor()).round() as i64;
    let (mut bed_low, mut bed_high) = ([i64::MAX; 2], [i64::MIN; 2]);
    for p in bed.iter().flat_map(Polygon::points) {
        for (a, v) in [p.x(), p.y()].into_iter().enumerate() {
            (bed_low[a], bed_high[a]) = (bed_low[a].min(v), bed_high[a].max(v));
        }
    }
    for a in 0..2 {
        low[a] = low[a].max(bed_low[a] + shrink);
        high[a] = high[a].min(bed_high[a] - shrink);
        if low[a] >= high[a] {
            return Ok(Vec::new());
        }
    }
    let mut path = route::path_to_wipe_tower(route_start, start_wipe_pos, [low, high], &bed, scale);
    path.pop();
    Ok(path)
}
