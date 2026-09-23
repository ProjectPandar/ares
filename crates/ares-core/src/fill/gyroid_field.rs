//! `marchsq::GyroidField` (`FillGyroid.cpp:30-72`) — the gyroid implicit scalar
//! field for the `gyroid_optimized` marching-squares path, plus
//! `compute_omega_factor` (`FillGyroid.cpp:234`) and `get_gyroid_polylines`
//! (`FillGyroid.cpp:83`). The `omega` factor shortens the vertical wavelength
//! for buckling resistance while keeping the XY period at baseline.

use crate::geometry::marching_squares::{self, Coord};
use crate::geometry::{BoundingBox, CoordinateScale, Point, Polyline};

/// `gsizef`/`rsizef` (`FillGyroid.cpp:33-34`) — raster cell/gsize in mm.
const GSIZE_F: f64 = 0.40;
const RSIZE_F: f64 = 0.004;

/// `compute_omega_factor` (`FillGyroid.cpp:234-240`).
fn compute_omega_factor(density_adjusted: f64, line_spacing: f64, layer_height: f64) -> f64 {
    let lh_ratio = if line_spacing > 0.0 {
        layer_height / line_spacing
    } else {
        0.5
    };
    let correction = 1.0 / (1.0 + lh_ratio).sqrt();
    let raw = (1.0 / density_adjusted.max(0.1)).sqrt() * correction;
    raw.clamp(1.0, 2.0)
}

/// `marchsq::GyroidField` — the `sin(fx·x)cos(fy·y)+sin(fy·y)cos(fz·z)+
/// sin(fz·z)cos(fx·x)` field sampled on an `rsize` raster.
pub(super) struct GyroidField {
    rsize: i64,
    gsize: i64,
    offs: Point,
    size: Point,
    z: f64,
    fx: f32,
    fy: f32,
    fz: f32,
    scale: CoordinateScale,
}

impl GyroidField {
    /// `GyroidField(bb, z, period, omega)` — `fx=fy=baseline`, `fz=omega*baseline`
    /// where `baseline = float(2π) / max(period, 1e-3f)` (FillGyroid.cpp:44-54).
    pub(super) fn new(
        bb: BoundingBox,
        z: f64,
        period: f32,
        omega: f32,
        scale: CoordinateScale,
    ) -> Self {
        let baseline = 2.0_f32 * std::f32::consts::PI / period.max(1e-3);
        Self {
            rsize: scale.checked_scale(RSIZE_F).unwrap_or(1),
            gsize: (GSIZE_F / RSIZE_F).round() as i64,
            offs: bb.min(),
            size: Point::new(bb.max().x() - bb.min().x(), bb.max().y() - bb.min().y()),
            z,
            fx: baseline,
            fy: baseline,
            fz: omega * baseline,
            scale,
        }
    }

    /// `get_scalar(x,y,z)` — all `float` arithmetic (`FillGyroid.cpp:56-62`).
    fn get_scalar(&self, x: f64, y: f64, z: f64) -> f32 {
        let a = self.fx * x as f32;
        let b = self.fy * y as f32;
        let c = self.fz * z as f32;
        a.sin() * b.cos() + b.sin() * c.cos() + c.sin() * a.cos()
    }

    fn to_point(&self, c: Coord) -> Point {
        Point::new(
            c.c * self.rsize + self.offs.x(),
            c.r * self.rsize + self.offs.y(),
        )
    }

    fn to_pointf(&self, c: Coord) -> (f64, f64) {
        let p = self.to_point(c);
        (self.scale.unscale(p.x()), self.scale.unscale(p.y()))
    }

    fn value_at(&self, row: i64, col: i64) -> f32 {
        let (x, y) = self.to_pointf(Coord { r: row, c: col });
        self.get_scalar(x, y, self.z)
    }

    fn rows(&self) -> i64 {
        self.size.y() / self.rsize
    }
    fn cols(&self) -> i64 {
        self.size.x() / self.rsize
    }
}

/// `get_gyroid_polylines` (`FillGyroid.cpp:83-98`) — execute the marching
/// squares at `isoval=0`, close each ring, Douglas-Peucker simplify at
/// `SCALED_SPARSE_INFILL_RESOLUTION`.
pub(super) fn get_gyroid_polylines(field: &GyroidField, scale: CoordinateScale) -> Vec<Polyline> {
    let rings = marching_squares::execute(
        |r, c| field.value_at(r, c),
        field.rows(),
        field.cols(),
        0.0,
        Coord {
            r: field.gsize,
            c: field.gsize,
        },
    );
    rings
        .into_iter()
        .map(|ring| {
            let mut pts: Vec<Point> = ring.iter().map(|&c| field.to_point(c)).collect();
            if let Some(&first) = pts.first() {
                pts.push(first);
            }
            Polyline::new(crate::geometry::simplification::douglas_peucker(
                &pts,
                super::tpms_fk::SPARSE_INFILL_RESOLUTION / scale.factor(),
            ))
        })
        .collect()
}

/// `compute_omega_factor` re-exported for the gyroid fill dispatch.
pub(super) fn omega_factor(density_adjusted: f64, line_spacing: f64, layer_height: f64) -> f64 {
    compute_omega_factor(density_adjusted, line_spacing, layer_height)
}
