//! `marchsq::AdaptiveTpmsField`, `make_adaptive_tpms` (`AdaptiveInfillField.cpp:636-708,751-776`),
//! and continuous adaptive `FillGyroid` (`FillGyroid.cpp:25-28,311-395`).

use crate::{
    ProcessAdaptiveInfillGradient, ProcessAdaptiveInfillMode,
    fill::{
        checked_rotate::{rotate_points, rotate_points_with_trig},
        connect::{FillConnectionParams, connect_infill},
        join_close_ends::join_close_ends,
        multiline_offset,
    },
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, Polyline,
        intersection_open_polylines,
        marching_squares::{self, Coord},
        offset_expolygon,
        simplification::douglas_peucker,
    },
};

use super::{
    AdaptiveInfillField, AdaptiveTpms, Radial, Radials,
    scale::{DensityLevels, RadialScale},
};

const BLEND_RATIO: f64 = 2.5;
const GSIZE_F: f32 = 0.40;
const RSIZE_F: f32 = 0.004;
const SPARSE_INFILL_RESOLUTION: f64 = 0.04;
const EPSILON: f64 = 1.0e-4;
const DENSITY_ADJUST: f64 = 2.44;
const GYROID_ANGLE_OFFSET: f64 = -std::f64::consts::FRAC_PI_4;
const PI: f64 = std::f64::consts::PI;

#[derive(Clone, Copy)]
pub(crate) struct AdaptiveGyroidParams {
    pub(crate) z: f64,
    pub(crate) spacing: f64,
    pub(crate) overlap: f64,
    pub(crate) angle: f32,
    pub(crate) density: f32,
    pub(crate) interior_density: f32,
    pub(crate) gradient: ProcessAdaptiveInfillGradient,
    pub(crate) multiline: i32,
    pub(crate) layer_height: f64,
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) dont_sort: bool,
}

/// `FillGyroid.cpp:25-28`: standard implicit gyroid equation in `f32`.
fn gyroid_equation(x: f32, y: f32, z: f32) -> f32 {
    x.sin() * y.cos() + y.sin() * z.cos() + z.sin() * x.cos()
}

struct AdaptiveTpmsField<'a> {
    rsize: i64,
    gsize: i64,
    tpms: AdaptiveTpms,
    radial_field: &'a AdaptiveInfillField,
    scale: RadialScale,
    levels: DensityLevels,
    size: Point,
    offs: Point,
    z: f64,
    cos_angle: f64,
    sin_angle: f64,
    coord_scale: CoordinateScale,
}

impl<'a> AdaptiveTpmsField<'a> {
    fn new(
        tpms: AdaptiveTpms,
        radial_field: &'a AdaptiveInfillField,
        bbox: BoundingBox,
        (z, angle): (f64, f32),
        coord_scale: CoordinateScale,
    ) -> Self {
        let rsize = coord_scale.checked_scale(f64::from(RSIZE_F)).unwrap_or(1);
        let gsize = (GSIZE_F / RSIZE_F).round() as i64;
        let dimensions = if radial_field.axis() < 0 { 3 } else { 2 };
        let ratio = (tpms.interior_frequency / tpms.surface_frequency).max(1e-3);
        Self {
            rsize,
            gsize,
            tpms,
            radial_field,
            scale: RadialScale::new(tpms, dimensions),
            levels: DensityLevels::new(ratio, BLEND_RATIO, tpms.gradient),
            size: Point::new(
                bbox.max().x() - bbox.min().x(),
                bbox.max().y() - bbox.min().y(),
            ),
            offs: bbox.min(),
            z,
            // Upstream calls std::cos(float)/std::sin(float) then widens to double.
            cos_angle: f64::from(angle.cos()),
            sin_angle: f64::from(angle.sin()),
            coord_scale,
        }
    }

    fn to_point(&self, p: Coord) -> Point {
        Point::new(
            p.c * self.rsize + self.offs.x(),
            p.r * self.rsize + self.offs.y(),
        )
    }

    fn rows(&self) -> i64 {
        self.size.y() / self.rsize
    }

    fn cols(&self) -> i64 {
        self.size.x() / self.rsize
    }

    fn get_scalar(&self, p: Coord) -> f32 {
        let pt = self.to_point(p);
        let x = self.coord_scale.unscale(pt.x());
        let y = self.coord_scale.unscale(pt.y());
        let obj = [
            self.cos_angle * x - self.sin_angle * y,
            self.sin_angle * x + self.cos_angle * y,
            self.z,
        ];
        if self.radial_field.mode() == ProcessAdaptiveInfillMode::SmoothBlend {
            let lattice = |level: i32| -> f32 {
                let frequency = self.tpms.surface_frequency * self.levels.scale(level);
                (self.tpms.equation)(
                    (frequency * x) as f32,
                    (frequency * y) as f32,
                    (frequency * self.z) as f32,
                )
            };
            if self.levels.count == 0 {
                return lattice(0);
            }
            let c = self
                .levels
                .level(self.radial_field.depth(obj))
                .clamp(0.0, f64::from(self.levels.count));
            let k = (c as i32).min(self.levels.count - 1);
            let u = c - f64::from(k);
            let w = u * u * (3.0 - 2.0 * u);
            let low = if w < 1.0 {
                (1.0 - w) * f64::from(lattice(k))
            } else {
                0.0
            };
            let high = if w > 0.0 {
                w * f64::from(lattice(k + 1))
            } else {
                0.0
            };
            return (low + high) as f32;
        }
        let axis = self.radial_field.axis();
        let mut radials: Radials = [Radial::default(); 8];
        let count = self.radial_field.radial(obj, &mut radials);
        let mut value = 0.0_f32;
        for radial in &radials[..count] {
            let factor = self.tpms.surface_frequency * self.scale.evaluate(radial.t);
            let mut q = [
                factor * (obj[0] - radial.center[0]),
                factor * (obj[1] - radial.center[1]),
                factor * (obj[2] - radial.center[2]),
            ];
            if axis >= 0 {
                let a = axis as usize;
                q[a] = self.tpms.interior_frequency * obj[a];
            }
            let qx = (self.cos_angle * q[0] + self.sin_angle * q[1]) as f32;
            let qy = (self.cos_angle * q[1] - self.sin_angle * q[0]) as f32;
            value += radial.weight * (self.tpms.equation)(qx, qy, q[2] as f32);
        }
        value
    }
}

pub(crate) fn make_adaptive_tpms(
    tpms: AdaptiveTpms,
    field: &AdaptiveInfillField,
    mut bbox: BoundingBox,
    (z, layer_height, spacing, angle): (f64, f64, f64, f32),
    scale: CoordinateScale,
) -> Vec<Polyline> {
    let cell = scale.checked_scale(f64::from(GSIZE_F)).unwrap_or(1);
    bbox.offset(cell);
    let aligned = Point::new(
        bbox.min().x().div_euclid(cell) * cell,
        bbox.min().y().div_euclid(cell) * cell,
    );
    bbox.merge_point(aligned);

    let raster = AdaptiveTpmsField::new(tpms, field, bbox, (z - 0.5 * layer_height, angle), scale);
    let rings = marching_squares::execute(
        |r, c| raster.get_scalar(Coord { r, c }),
        raster.rows(),
        raster.cols(),
        0.0,
        Coord {
            r: raster.gsize,
            c: raster.gsize,
        },
    );

    let min_loop_length = 2.0 * PI * spacing / scale.factor();
    let tolerance = SPARSE_INFILL_RESOLUTION / scale.factor();
    let mut polylines = Vec::with_capacity(rings.len());
    for ring in rings {
        let mut pts: Vec<Point> = ring.iter().map(|&crd| raster.to_point(crd)).collect();
        if let Some(&first) = pts.first() {
            pts.push(first);
        }
        let simplified = Polyline::new(douglas_peucker(&pts, tolerance));
        if polyline_length(&simplified) >= min_loop_length {
            polylines.push(simplified);
        }
    }
    polylines
}

pub(crate) fn fill_adaptive_gyroid(
    surface: &ExPolygon,
    params: AdaptiveGyroidParams,
    field: &AdaptiveInfillField,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let offset = ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32;
    let components = offset_expolygon(surface, offset, JoinType::Miter, 3.0)?;
    let mut output = Vec::new();
    for component in components {
        output.extend(fill_adaptive_gyroid_component(
            &component, params, field, scale,
        )?);
    }
    Ok(output)
}

fn fill_adaptive_gyroid_component(
    surface: &ExPolygon,
    params: AdaptiveGyroidParams,
    field: &AdaptiveInfillField,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let infill_angle_f32 = (f64::from(params.angle) + GYROID_ANGLE_OFFSET) as f32;
    let infill_angle = f64::from(infill_angle_f32);
    let rotate = infill_angle.abs() >= EPSILON;
    let rotated = if rotate {
        rotate_expolygon(surface, -infill_angle)?
    } else {
        surface.clone()
    };
    let Some(mut bb) = BoundingBox::from_polygon(rotated.contour()) else {
        return Ok(Vec::new());
    };
    bb.offset((params.spacing * f64::from(params.multiline) / scale.factor()) as i64);

    let frequency = |density: f32| -> f64 {
        f64::from(density) * DENSITY_ADJUST / (f64::from(params.multiline) * params.spacing)
    };
    let tpms = AdaptiveTpms {
        equation: gyroid_equation,
        surface_frequency: frequency(params.density),
        interior_frequency: frequency(params.interior_density),
        gradient: params.gradient,
    };
    let mut polylines = make_adaptive_tpms(
        tpms,
        field,
        bb,
        (
            params.z,
            params.layer_height,
            params.spacing,
            infill_angle_f32,
        ),
        scale,
    );
    polylines = multiline_offset::apply(polylines, params.multiline, params.spacing, scale)?;

    let (contour, holes) = rotated.clone().into_parts();
    let mut clip = Vec::with_capacity(holes.len() + 1);
    clip.push(contour);
    clip.extend(holes);
    polylines = intersection_open_polylines(&polylines, &clip)?;
    polylines = join_close_ends(
        polylines,
        EPSILON / scale.factor(),
        true,
        EPSILON / scale.factor(),
    );
    let minimum_length = 0.8 * params.spacing / scale.factor();
    polylines.retain(|polyline| polyline_length(polyline) >= minimum_length);
    if polylines.is_empty() {
        return Ok(Vec::new());
    }
    let mut connected = connect_infill(
        polylines,
        &rotated,
        params.spacing,
        FillConnectionParams {
            anchor_length: params.anchor_length,
            anchor_length_max: params.anchor_length_max,
            multiline: params.multiline,
            dont_sort: params.dont_sort,
        },
        scale,
    )?;
    if rotate {
        rotate_polylines(&mut connected, infill_angle)?;
    }
    Ok(connected)
}

fn polyline_length(polyline: &Polyline) -> f64 {
    polyline
        .points()
        .windows(2)
        .map(|w| {
            let dx = (w[1].x() - w[0].x()) as f64;
            let dy = (w[1].y() - w[0].y()) as f64;
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

fn rotate_expolygon(expolygon: &ExPolygon, angle: f64) -> Result<ExPolygon, ClipperError> {
    let rotate =
        |polygon: &Polygon| rotate_points(polygon.points().to_vec(), angle).map(Polygon::new);
    Ok(ExPolygon::new(
        rotate(expolygon.contour())?,
        expolygon
            .holes()
            .iter()
            .map(rotate)
            .collect::<Result<_, _>>()?,
    ))
}

fn rotate_polylines(polylines: &mut [Polyline], angle: f64) -> Result<(), ClipperError> {
    let (cosine, sine) = (angle.cos(), angle.sin());
    for polyline in polylines {
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(rotate_points_with_trig(points, cosine, sine)?);
    }
    Ok(())
}
