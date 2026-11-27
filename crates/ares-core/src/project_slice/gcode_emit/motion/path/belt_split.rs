//! GCode::belt_cut_at_band_edges, in scaled slicing coordinates.

use super::super::belt_bands::BeltBands;
use super::{EmitState, LayerGeometry, PathProperties, arc, emit_piece};
use crate::geometry::CoordinateScale;

type Point = (i64, i64);
type PathRequest<'a> = (PathProperties<'a>, Option<f64>, bool);
pub(super) type PieceEmission<'a> = (PathProperties<'a>, Option<f64>, bool, &'a [f64]);

struct Piece {
    points: Vec<Point>,
    uncut_lengths: Vec<f64>,
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    points: Vec<Point>,
    (properties, speed, no_extrusion): PathRequest<'_>,
    geometry: LayerGeometry<'_>,
    (bands, state): (BeltBands, &mut EmitState),
) -> Result<Option<Vec<arc::Point>>, crate::SliceError> {
    let pieces = split(
        &points,
        bands,
        (geometry.scale, properties.width),
        state.options.slow_down_layers,
    );
    if pieces.len() < 2 {
        state.belt_piece = Some(classify(&points, bands, (geometry.scale, properties.width)));
        let result = emit_piece(
            output,
            points,
            (properties, speed, no_extrusion, &[]),
            geometry,
            state,
        );
        state.belt_piece = None;
        return result;
    }
    let properties = PathProperties {
        end_clip: 0.,
        ..properties
    };
    for Piece {
        points,
        uncut_lengths,
    } in pieces
    {
        state.belt_piece = Some(classify(&points, bands, (geometry.scale, properties.width)));
        let result = emit_piece(
            output,
            points,
            (properties, speed, no_extrusion, &uncut_lengths),
            geometry,
            state,
        );
        state.belt_piece = None;
        result?;
    }
    Ok(Some(
        points
            .into_iter()
            .rev()
            .map(|(x, y)| arc::Point {
                x: geometry.scale.unscale(x),
                y: geometry.scale.unscale(y),
            })
            .collect(),
    ))
}

fn classify(
    points: &[Point],
    bands: BeltBands,
    (scale, width): (CoordinateScale, f32),
) -> (bool, usize) {
    let (lo, hi) = points
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &p| {
            let h = bands.height(p, scale, width);
            (lo.min(h), hi.max(h))
        });
    let first = bands.is_first(hi);
    (
        first,
        if first {
            0
        } else {
            bands.index(0.5 * (lo + hi)).max(1)
        },
    )
}

fn split(
    points: &[Point],
    bands: BeltBands,
    (scale, width): (CoordinateScale, f32),
    slowdown_layers: u32,
) -> Vec<Piece> {
    let band_of = |h| {
        if bands.is_first(h) {
            0
        } else {
            bands.index(h).clamp(1, slowdown_layers.max(1) as usize)
        }
    };
    let band_top = |k| {
        if k == 0 {
            bands.first
        } else {
            (k + 1) as f64 * bands.thickness()
        }
    };
    let mut pieces = Vec::new();
    let mut current = Piece {
        points: vec![points[0]],
        uncut_lengths: Vec::new(),
    };
    let mut prev_h = bands.height(points[0], scale, width);
    let mut prev_band = band_of(prev_h);
    for pair in points.windows(2) {
        let [a, b] = [pair[0], pair[1]];
        let h = bands.height(b, scale, width);
        let band = band_of(h);
        let dx = (b.0 - a.0) as f64;
        let dy = (b.1 - a.1) as f64;
        let length = (dx * dx + dy * dy).sqrt() * scale.factor();
        while prev_band != band {
            let edge = if band > prev_band {
                prev_band += 1;
                band_top(prev_band - 1)
            } else {
                prev_band -= 1;
                band_top(prev_band)
            };
            let t = ((edge - prev_h) / (h - prev_h)).clamp(0., 1.);
            // GCode.cpp constructs a Point3 here; Point.hpp rounds each coordinate.
            let cut = (
                (a.0 as f64 + t * dx).round() as i64,
                (a.1 as f64 + t * dy).round() as i64,
            );
            current.push(cut, length);
            let next = Piece {
                points: vec![cut],
                uncut_lengths: Vec::new(),
            };
            if current.points.len() > 1 {
                pieces.push(std::mem::replace(&mut current, next));
            } else {
                current = next;
            }
        }
        current.push(b, length);
        prev_h = h;
    }
    if !pieces.is_empty() && current.points.len() > 1 {
        pieces.push(current);
    }
    pieces
}

impl Piece {
    fn push(&mut self, point: Point, uncut: f64) {
        if self.points.last() != Some(&point) {
            self.points.push(point);
            self.uncut_lengths.push(uncut);
        }
    }
}
