//! `BeltBrim.cpp`: cross-belt lines and per-line bead dimensions.

use super::{Band, BandPath, BeltFloorContext, CoordinateScale, Frame, cutoff_band, geometry};
use crate::{
    geometry::{
        BoundingBox, ClipperError, ExPolygon, JoinType, Polygon, Polyline,
        difference_open_polylines, intersection_open_polylines, offset_open_paths,
        union_polygons_paths,
    },
    project_slice::{
        belt::EPSILON,
        chain_polylines,
        perimeters::{classic::traversal::PostClassicTraversalPrintObject, flow::ordinary_volume},
    },
};

pub(super) struct Context {
    pub(super) floor: BeltFloorContext,
    pub(super) frame: Frame,
    pub(super) bounds: BoundingBox,
    pub(super) scale: CoordinateScale,
    pub(super) pitch: i64,
    pub(super) anchor: i64,
    pub(super) region: Vec<Polygon>,
}

impl Context {
    pub(super) fn band(
        &self,
        object: &PostClassicTraversalPrintObject,
        (print_z, height): (f64, f64),
        inset: f64,
    ) -> Result<Band, ClipperError> {
        let mut band = Band {
            print_z,
            height,
            paths: Vec::new(),
            areas: Vec::new(),
        };
        let scale = self.scale;
        let scaled = |u| scale.scaled_delta(u) as i64;
        let (lo, hi) = cutoff_band(self.floor, (print_z, height));
        let stretch = self.frame.stretch();
        let nominal_pitch = scale.unscale(self.pitch) * stretch;
        let band_pitch = (hi - lo) * stretch;
        let wall_clearance = (0.5 * height + inset * self.frame.shear.abs()).max(0.1 * height);
        let steep = band_pitch <= 1.5 * nominal_pitch + EPSILON;
        let line_pitch = if steep { band_pitch } else { nominal_pitch };
        let wall_u = scaled(self.floor.cutoff_u(print_z - wall_clearance));
        let first_u = if steep {
            wall_u
        } else {
            let lo = scaled(lo);
            lo + (self.anchor - lo).rem_euclid(self.pitch)
        };
        let end_u = if steep { wall_u + 1 } else { scaled(hi) };
        let obstacles = self.obstacles(object, (print_z, 0.5 * height))?;
        let margin = scaled(EPSILON) + 1;
        let v0 = self.frame.v(self.bounds.min()) - margin;
        let v1 = self.frame.v(self.bounds.max()) + margin;
        let mut previous = None::<i64>;
        let mut lattice = first_u;
        while lattice < end_u {
            let u = lattice;
            lattice += self.pitch;
            let clearance = if steep {
                wall_clearance
            } else {
                (print_z - self.floor.floor_print_z(scale.unscale(u))).min(height)
            };
            let (u, clearance) = if clearance < wall_clearance {
                (wall_u, wall_clearance)
            } else {
                (u, clearance)
            };
            if previous.is_some_and(|prev| u.abs_diff(prev) < (self.pitch / 2) as u64) {
                continue;
            }
            previous = Some(u);
            let line = Polyline::new(vec![self.frame.uv(u, v0), self.frame.uv(u, v1)]);
            let mut pieces = intersection_open_polylines(&[line], &self.region)?;
            if !obstacles.is_empty() {
                pieces = difference_open_polylines(&pieces, &obstacles)?;
            }
            let height = clearance as f32;
            let width = (f64::from(line_pitch as f32)
                + f64::from(height) * (1.0 - 0.25 * std::f64::consts::PI))
                as f32;
            let bead_lines = pieces
                .iter()
                .map(|p| Polygon::new(p.points().to_vec()))
                .collect::<Vec<_>>();
            let radius = 0.5 * scaled(f64::from(width)) as f32;
            band.areas.extend(
                offset_open_paths(&bead_lines, radius, JoinType::Square, 0.0)?
                    .into_iter()
                    .map(|p| ExPolygon::new(p, Vec::new())),
            );
            chain_polylines(&mut pieces, scale);
            band.paths
                .extend(pieces.into_iter().map(|polyline| BandPath {
                    polyline,
                    mm3_per_mm: ordinary_volume(width, height),
                    width,
                    height,
                }));
        }
        Ok(band)
    }

    fn obstacles(
        &self,
        object: &PostClassicTraversalPrintObject,
        (print_z, tolerance): (f64, f64),
    ) -> Result<Vec<Polygon>, ClipperError> {
        let layers = object.physical_layers();
        let index = layers.partition_point(|layer| layer.print_z < print_z - tolerance);
        if layers
            .get(index)
            .is_none_or(|layer| layer.print_z > print_z + tolerance)
        {
            return Ok(Vec::new());
        }
        let slices = object.slices(index).unwrap_or_default();
        if !BoundingBox::from_expolygons(slices).is_some_and(|bounds| bounds.overlap(self.bounds)) {
            return Ok(Vec::new());
        }
        let paths = geometry::rings(slices).cloned().collect::<Vec<_>>();
        if paths.len() < 2 {
            Ok(paths)
        } else {
            union_polygons_paths(&paths)
        }
    }
}
