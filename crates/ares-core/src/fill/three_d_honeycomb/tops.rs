//! `Fill3DHoneycomb.cpp:92–168` complete-top covers.

use super::{
    EPSILON,
    grid::{Grid, cycle, point},
    wave::{tri, troct},
};
use crate::geometry::{Point, Polyline};

struct Cover {
    start_l: f64,
    end_l: f64,
    start_p: f64,
    end_p: f64,
    spacing: f64,
}

pub(super) fn add(grid: Grid, distance: usize) -> Vec<Polyline> {
    let vertical = cycle(grid.z, grid.size) < 0.5;
    let cover = cover(grid, distance);
    let Some(cover) = cover else {
        return Vec::new();
    };
    let make_cover = if vertical == (distance % 2 == 1) {
        vertical_cover
    } else {
        horizontal_cover
    };
    let mut lines = Vec::new();
    let mut x = grid.multiline as f64;
    let mut xm = 0;
    while x <= grid.width {
        let mut y = grid.multiline as f64;
        let mut ym = 0;
        while y <= grid.height {
            if ((xm ^ ym) == 1) != vertical {
                let points = make_cover(grid, &cover, x, y, xm ^ ym);
                lines.push(Polyline::new(points));
            }
            y += grid.size;
            ym ^= 1;
        }
        x += grid.size;
        xm ^= 1;
    }
    lines
}

fn cover(grid: Grid, distance: usize) -> Option<Cover> {
    let perpendicular = (tri(grid.z, grid.size) / 2.0).abs();
    let grid_point = grid.size * (perpendicular / grid.size);
    let top_offset = grid.size / 2.0 - troct(grid_point, grid.size, grid.z).abs();
    let mut start_l = grid.size * 0.5 - top_offset;
    let mut end_l = grid.size * 0.5 + top_offset;
    if distance == 0 && grid.multiline == 1 {
        start_l -= grid.spacing;
        end_l += grid.spacing;
    } else if grid.multiline > 1 {
        let adjustment = (2.0_f64.sqrt() - 1.0) / 2.0;
        start_l -= grid.spacing * grid.multiline as f64 * adjustment;
        end_l += grid.spacing * grid.multiline as f64 * adjustment;
    }
    let mut start_p = grid.size * 0.5 - top_offset
        + grid.spacing * grid.multiline as f64 / 2.0
        + grid.spacing / 2.0;
    let mut end_p = grid.size * 0.5 + top_offset
        - grid.spacing * grid.multiline as f64 / 2.0
        - grid.spacing / 2.0;
    if distance.is_multiple_of(2) {
        std::mem::swap(&mut start_l, &mut start_p);
        std::mem::swap(&mut end_l, &mut end_p);
    }
    let regions = ((end_p - start_p) / (grid.spacing / 2.0_f64.sqrt())).floor();
    if regions <= 0.0 {
        return None;
    }
    Some(Cover {
        start_l,
        end_l,
        start_p,
        end_p,
        spacing: (end_p - start_p) / regions,
    })
}

fn vertical_cover(grid: Grid, cover: &Cover, x: f64, y: f64, mut direction: i32) -> Vec<Point> {
    let mut points = Vec::new();
    if y < grid.height - cover.spacing * grid.multiline as f64 * 1.5 {
        let end_p = (grid.width - grid.multiline as f64 * (cover.spacing + 1.0) / 2.0)
            .min(x + cover.end_p)
            - x;
        let end_l = (grid.height - grid.multiline as f64 * (cover.spacing + 1.0) / 2.0)
            .min(y + cover.end_l)
            - y;
        let mut xi = cover.start_p;
        while xi < end_p + EPSILON {
            points.push(if direction == 0 {
                point(x + xi, y + cover.start_l)
            } else {
                point(x + xi, y + end_l)
            });
            points.push(if direction == 0 {
                point(x + xi, y + end_l)
            } else {
                point(x + xi, y + cover.start_l)
            });
            xi += cover.spacing;
            direction ^= 1;
        }
    }
    points
}

fn horizontal_cover(grid: Grid, cover: &Cover, x: f64, y: f64, mut direction: i32) -> Vec<Point> {
    let mut points = Vec::new();
    if x < grid.width - cover.spacing * grid.multiline as f64 * 1.5 {
        let end_p = (grid.height - grid.multiline as f64 * (cover.spacing + 1.0) / 2.0)
            .min(y + cover.end_p)
            - y;
        let end_l = (grid.width - grid.multiline as f64 * (cover.spacing + 1.0) / 2.0)
            .min(x + cover.end_l)
            - x;
        let mut yi = cover.start_p;
        while yi < end_p + EPSILON {
            points.push(if direction == 0 {
                point(x + cover.start_l, y + yi)
            } else {
                point(x + end_l, y + yi)
            });
            points.push(if direction == 0 {
                point(x + end_l, y + yi)
            } else {
                point(x + cover.start_l, y + yi)
            });
            yi += cover.spacing;
            direction ^= 1;
        }
    }
    points
}
