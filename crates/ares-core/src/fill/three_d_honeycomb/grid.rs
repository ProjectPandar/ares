//! `Fill3DHoneycomb.cpp:174–324` ordered zigzags and end connectors.

use super::{
    EPSILON, tops,
    wave::{critical, sign, troct},
};
use crate::geometry::{Point, Polyline};

#[derive(Clone, Copy)]
pub(super) struct Grid {
    pub(super) z: f64,
    pub(super) z_last: f64,
    pub(super) size: f64,
    pub(super) width: f64,
    pub(super) height: f64,
    pub(super) complete_top: bool,
    pub(super) spacing: f64,
    pub(super) multiline: usize,
    pub(super) layers: usize,
}

pub(super) fn cycle(z: f64, size: f64) -> f64 {
    (z + size / 2.0) % (size * 2.0) / (size * 2.0)
}

// Point.hpp:208: the double constructor rounds the completed coordinates.
pub(super) fn point(x: f64, y: f64) -> Point {
    Point::new(x.round() as i64, y.round() as i64)
}

pub(super) fn translated(line: &Polyline, delta: Point) -> Polyline {
    Polyline::new(
        line.points()
            .iter()
            .map(|p| Point::new(p.x() + delta.x(), p.y() + delta.y()))
            .collect(),
    )
}

pub(super) fn make_grid(grid: Grid) -> Vec<Polyline> {
    let vertical = cycle(grid.z, grid.size) < 0.5;
    let last_vertical = cycle(grid.z_last, grid.size) < 0.5;
    let mut result = zigzags(grid);
    let critical = critical(grid.z, grid.size);
    result.extend(endpoints(grid, &critical));
    if grid.complete_top && vertical != last_vertical {
        let layer_height = (grid.z - grid.z_last) / (grid.multiline * grid.layers) as f64;
        let mut distance = 0;
        let mut z_check = grid.z;
        while z_check >= grid.z_last + EPSILON {
            if vertical != (cycle(z_check, grid.size) < 0.5) {
                break;
            }
            z_check -= layer_height * grid.layers as f64;
            distance += 1;
        }
        result.extend(tops::add(grid, distance));
    }
    result
}

fn zigzags(grid: Grid) -> Vec<Polyline> {
    let critical = critical(grid.z, grid.size);
    let vertical = cycle(grid.z, grid.size) < 0.5;
    let maximum = (((if vertical { grid.width } else { grid.height }) - EPSILON) / grid.size)
        .ceil()
        * grid.size;
    let mut lines = Vec::new();
    let mut direction = -1;
    let mut pattern_direction = -1;
    let mut position = 0.0;
    while position < maximum {
        for index in 0..grid.multiline {
            let adjustment = (index as f64 - (grid.multiline - 1) as f64 / 2.0) * grid.spacing;
            let mut points = pattern(
                grid,
                &critical,
                pattern_direction,
                if vertical { 1 } else { -1 },
                adjustment,
            );
            if direction == 1 {
                points.reverse();
            }
            let line = Polyline::new(points);
            lines.push(translated(
                &line,
                if vertical {
                    point(position + adjustment, 0.0)
                } else {
                    point(0.0, position + adjustment)
                },
            ));
            direction *= -1;
        }
        position += grid.size;
        pattern_direction *= -1;
    }
    lines
}

fn pattern(
    grid: Grid,
    critical: &[f64],
    perpendicular: i32,
    direction: i32,
    adjustment: f64,
) -> Vec<Point> {
    let diagonal = adjustment * (2.0_f64.sqrt() - 1.0);
    let vertical = direction == 1;
    let z_flip = sign(cycle(grid.z, grid.size) % 0.5 - 0.25);
    let length = if vertical { grid.height } else { grid.width };
    let maximum = ((length - EPSILON) / grid.size).ceil() * grid.size;
    let mut points = vec![if vertical {
        point(diagonal, 0.0)
    } else {
        point(0.0, diagonal)
    }];
    let mut offset = 0;
    let mut location = 0.0;
    while location < maximum {
        for (within, &critical) in critical[offset..offset + 2].iter().enumerate() {
            let index = offset + within;
            let waveform = troct(critical, grid.size, grid.z);
            let position_flip = (((index + 1) % 4) / 2) as f64 * 2.0 - 1.0;
            let linear = location - (offset as f64 * grid.size / 2.0) + critical;
            let across = waveform * f64::from(perpendicular);
            let along = linear
                + position_flip
                    * diagonal
                    * f64::from(perpendicular)
                    * z_flip
                    * f64::from(direction);
            points.push(if vertical {
                point(across, along)
            } else {
                point(along, across)
            });
        }
        location += grid.size;
        offset = (offset + 2) % 4;
    }
    points.push(if vertical {
        point(diagonal, maximum)
    } else {
        point(maximum, diagonal)
    });
    points
}

fn endpoints(grid: Grid, critical: &[f64]) -> Vec<Polyline> {
    let vertical = cycle(grid.z, grid.size) < 0.5;
    let horizontal = cycle(grid.z, grid.size) >= 0.5;
    let flip = usize::from((sign(cycle(grid.z, grid.size) % 0.5 - 0.25) > 0.0) != vertical);
    let mut starts = Vec::new();
    let mut ends = Vec::new();
    for index in 0..grid.multiline {
        let adjustment = (index as f64 - (grid.multiline - 1) as f64 / 2.0) * grid.spacing;
        let diagonal = adjustment * 2.0_f64.sqrt();
        let mut start = vec![if horizontal {
            point(adjustment, -diagonal)
        } else {
            point(-diagonal, adjustment)
        }];
        let mut end = vec![if horizontal {
            point(-adjustment, -diagonal)
        } else {
            point(-diagonal, -adjustment)
        }];
        for (index, &critical) in critical.iter().take(2).enumerate() {
            let direction = index as i32 * 2 - 1;
            let offset = f64::from(direction) * (2.0_f64.sqrt() - 1.0) * adjustment;
            let waveform = troct(critical, grid.size, grid.z).abs();
            start.push(if horizontal {
                point(-waveform, critical + offset)
            } else {
                point(critical + offset, -waveform)
            });
            end.push(if horizontal {
                point(waveform, critical + offset)
            } else {
                point(critical + offset, waveform)
            });
        }
        start.push(if horizontal {
            point(adjustment, grid.size + diagonal)
        } else {
            point(grid.size + diagonal, adjustment)
        });
        end.push(if horizontal {
            point(-adjustment, grid.size + diagonal)
        } else {
            point(grid.size + diagonal, -adjustment)
        });
        starts.push(Polyline::new(start));
        ends.push(Polyline::new(end));
    }
    let max_x = ((grid.width - EPSILON) / grid.size).ceil() * grid.size;
    let max_y = ((grid.height - EPSILON) / grid.size).ceil() * grid.size;
    let mut lines = Vec::new();
    for index in 0..grid.multiline {
        let adjustment = (index as f64 - (grid.multiline - 1) as f64 / 2.0) * grid.spacing;
        let mut location = flip as f64 * grid.size;
        while location < (if horizontal { max_y } else { max_x }) - EPSILON {
            lines.push(translated(
                &starts[index],
                if vertical {
                    point(location, -adjustment)
                } else {
                    point(-adjustment, location)
                },
            ));
            lines.push(translated(
                &ends[index],
                if vertical {
                    point(location, max_y + adjustment)
                } else {
                    point(max_x + adjustment, location)
                },
            ));
            location += grid.size * 2.0;
        }
    }
    lines
}
