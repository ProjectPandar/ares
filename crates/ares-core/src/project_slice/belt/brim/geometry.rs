//! `BeltBrim.cpp`: flattening, swept brim rings and contact boxes.

use crate::{
    ProcessBrimType,
    geometry::{
        BoundingBox, ClipperError, ExPolygon, JoinType, Point, Polygon, closing_ex, difference_ex,
        intersection_ex_polygons, offset_expolygons, offset_paths_tree, union_ex, union_expolygons,
    },
};

#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub(super) axis: usize,
    pub(super) shear: f64,
}

impl Frame {
    pub(super) fn stretch(self) -> f64 {
        (1.0 + self.shear * self.shear).sqrt()
    }

    pub(super) fn low_side(self) -> bool {
        self.shear > 0.0
    }

    pub(super) fn uv(self, u: i64, v: i64) -> Point {
        if self.axis == 0 {
            Point::new(u, v)
        } else {
            Point::new(v, u)
        }
    }

    pub(super) fn u(self, p: Point) -> i64 {
        [p.x(), p.y()][self.axis]
    }

    pub(super) fn v(self, p: Point) -> i64 {
        [p.y(), p.x()][self.axis]
    }

    pub(super) fn leading_u(self, bounds: BoundingBox) -> i64 {
        self.u(if self.low_side() {
            bounds.min()
        } else {
            bounds.max()
        })
    }

    pub(super) fn band_box(
        self,
        bounds: BoundingBox,
        (lo, hi): (i64, i64),
        margin: i64,
    ) -> Option<Polygon> {
        let lo = lo.max(self.u(bounds.min()) - margin);
        let hi = hi.min(self.u(bounds.max()) + margin);
        let v0 = self.v(bounds.min()) - margin;
        let v1 = self.v(bounds.max()) + margin;
        (lo < hi).then(|| {
            Polygon::new(if self.axis == 0 {
                vec![
                    self.uv(lo, v0),
                    self.uv(hi, v0),
                    self.uv(hi, v1),
                    self.uv(lo, v1),
                ]
            } else {
                vec![
                    self.uv(lo, v0),
                    self.uv(lo, v1),
                    self.uv(hi, v1),
                    self.uv(hi, v0),
                ]
            })
        })
    }

    pub(super) fn scale_u(self, polygons: Vec<ExPolygon>, factor: f64) -> Vec<ExPolygon> {
        let scale_polygon = |mut polygon: Polygon| {
            for p in polygon.points_mut() {
                *p = self.uv((self.u(*p) as f64 * factor).round() as i64, self.v(*p));
            }
            polygon
        };
        polygons
            .into_iter()
            .map(|p| {
                let (contour, holes) = p.into_parts();
                ExPolygon::new(
                    scale_polygon(contour),
                    holes.into_iter().map(scale_polygon).collect(),
                )
            })
            .collect()
    }
}

pub(super) fn rings(polygons: &[ExPolygon]) -> impl Iterator<Item = &Polygon> {
    polygons
        .iter()
        .flat_map(|p| std::iter::once(p.contour()).chain(p.holes()))
}

fn sweep(polygons: &[ExPolygon], t: Point) -> Result<Vec<ExPolygon>, ClipperError> {
    let shift = |p: Point| Point::new(p.x() + t.x(), p.y() + t.y());
    let shifted_ring = |p: &Polygon| Polygon::new(p.points().iter().copied().map(shift).collect());
    let shifted = polygons
        .iter()
        .map(|p| {
            ExPolygon::new(
                shifted_ring(p.contour()),
                p.holes().iter().map(shifted_ring).collect(),
            )
        })
        .collect::<Vec<_>>();
    let mut swept = rings(&union_expolygons(polygons, &shifted)?)
        .cloned()
        .collect::<Vec<_>>();
    for polygon in rings(polygons) {
        let points = polygon.points();
        for (&a, &b) in points.iter().zip(points.iter().cycle().skip(1)) {
            if a == b {
                continue;
            }
            let mut quad = Polygon::new(vec![a, b, shift(b), shift(a)]);
            if !quad.is_counter_clockwise() {
                quad.reverse();
            }
            swept.push(quad);
        }
    }
    union_ex(&swept, crate::geometry::FillRule::NonZero)
}

pub(super) fn region(
    frame: Frame,
    footprint: Vec<ExPolygon>,
    kind: ProcessBrimType,
    (width, gap, leading, lateral): (i64, i64, i64, i64),
    resolution: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let stretch = frame.stretch();
    let flat = frame.scale_u(footprint, stretch);
    let mut out = Vec::new();
    if kind != ProcessBrimType::InnerOnly {
        let contours = flat.iter().map(|p| p.contour().clone()).collect::<Vec<_>>();
        let inner = offset_paths_tree(&contours, gap as f32, JoinType::Round, resolution)?
            .into_expolygons(crate::geometry::CoordinateScale::Normal);
        let envelope = if width > 0 {
            closing_ex(&inner, width as f32, JoinType::Miter, 3.0)?
        } else {
            inner
        };
        let lead = if frame.low_side() { -leading } else { leading };
        let base = if leading > 0 {
            union_expolygons(&envelope, &sweep(&envelope, frame.uv(lead, 0))?)?
        } else {
            envelope.clone()
        };
        let base = if lateral > 0 {
            let widened = union_expolygons(
                &sweep(&base, frame.uv(0, lateral))?,
                &sweep(&base, frame.uv(0, -lateral))?,
            )?;
            union_expolygons(&base, &widened)?
        } else {
            base
        };
        let outer = offset_expolygons(&base, width as f32, JoinType::Round, resolution)?;
        out.extend(difference_ex(&outer, &envelope)?);
    }
    if matches!(
        kind,
        ProcessBrimType::InnerOnly | ProcessBrimType::OuterAndInner
    ) {
        let holes = flat
            .iter()
            .flat_map(|p| p.holes())
            .map(|p| {
                let mut p = p.clone();
                p.reverse();
                p
            })
            .collect::<Vec<_>>();
        if !holes.is_empty() {
            let offset = |delta: i64| {
                Ok::<_, ClipperError>(
                    offset_paths_tree(&holes, delta as f32, JoinType::Miter, 3.0)?
                        .into_expolygons(crate::geometry::CoordinateScale::Normal),
                )
            };
            let inner = offset(-(width + gap))?;
            let outer = offset(-gap)?;
            out.extend(intersection_ex_polygons(
                &difference_ex(&outer, &inner)?,
                &holes,
            )?);
        }
    }
    Ok(frame.scale_u(union_expolygons(&out, &[])?, stretch.recip()))
}
