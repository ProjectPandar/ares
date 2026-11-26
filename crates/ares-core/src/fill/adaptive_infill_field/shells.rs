//! `marchsq::AdaptiveLevelField`, `make_adaptive_infill_shells`, and
//! `fill_adaptive_infill_shells` (`AdaptiveInfillField.cpp:47,711-745,778-821`).

use crate::{
    ProcessAdaptiveInfillGradient,
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Point, Polygon,
        difference_ex, intersection_ex,
        marching_squares::{self, Coord},
        offset_expolygons, union_ex,
    },
};

use super::{AdaptiveInfillField, scale::DensityLevels};

const SHELL_RATIO: f64 = 1.5;
const GSIZE_F: f32 = 0.5;
const RSIZE_F: f32 = 0.05;

#[derive(Clone, Copy)]
pub(crate) struct ShellFillSpec {
    pub(crate) z: f64,
    pub(crate) spacing: f64,
    pub(crate) surface_density: f32,
    pub(crate) interior_density: f32,
    pub(crate) gradient: ProcessAdaptiveInfillGradient,
}

pub(crate) struct AdaptiveInfillShell {
    pub(crate) density: f32,
    pub(crate) expolygons: Vec<ExPolygon>,
}

/// `marchsq::AdaptiveLevelField` (`AdaptiveInfillField.cpp:711-745`).
struct AdaptiveLevelField<'a> {
    rsize: i64,
    gsize: i64,
    field: &'a AdaptiveInfillField,
    levels: &'a DensityLevels,
    size: Point,
    offs: Point,
    z: f64,
    coord_scale: CoordinateScale,
}

impl<'a> AdaptiveLevelField<'a> {
    fn new(
        field: &'a AdaptiveInfillField,
        levels: &'a DensityLevels,
        bbox: BoundingBox,
        z: f64,
        coord_scale: CoordinateScale,
    ) -> Self {
        let rsize = ((RSIZE_F / (coord_scale.factor() as f32)) as i64).max(1);
        let gsize = (GSIZE_F / RSIZE_F).round() as i64;
        Self {
            rsize,
            gsize,
            field,
            levels,
            size: Point::new(
                bbox.max().x() - bbox.min().x(),
                bbox.max().y() - bbox.min().y(),
            ),
            offs: bbox.min(),
            z,
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
        let depth = self.field.depth([
            self.coord_scale.unscale(pt.x()),
            self.coord_scale.unscale(pt.y()),
            self.z,
        ]);
        self.levels.level(depth) as f32
    }
}

/// `make_adaptive_infill_shells` (`AdaptiveInfillField.cpp:778-809`).
pub(crate) fn make_adaptive_infill_shells(
    field: &AdaptiveInfillField,
    expolygon: &ExPolygon,
    spec: ShellFillSpec,
    scale: CoordinateScale,
) -> Result<Vec<AdaptiveInfillShell>, ClipperError> {
    // Upstream divides `float interior_density / float surface_density` before widening to double.
    let levels = DensityLevels::new(
        f64::from(spec.interior_density / spec.surface_density),
        SHELL_RATIO,
        spec.gradient,
    );
    if levels.count == 0 {
        return Ok(vec![AdaptiveInfillShell {
            density: spec.surface_density,
            expolygons: vec![expolygon.clone()],
        }]);
    }
    let cell = ((GSIZE_F / (scale.factor() as f32)) as i64).max(1);
    let Some(mut bbox) = BoundingBox::from_expolygon(expolygon) else {
        return Ok(Vec::new());
    };
    bbox.offset(cell);
    let aligned = Point::new(
        bbox.min().x().div_euclid(cell) * cell,
        bbox.min().y().div_euclid(cell) * cell,
    );
    bbox.merge_point(aligned);
    let raster = AdaptiveLevelField::new(field, &levels, bbox, spec.z, scale);

    let mut shells = Vec::new();
    let mut remaining = vec![expolygon.clone()];
    for level in 0..levels.count {
        if remaining.is_empty() {
            break;
        }
        let isoval = (f64::from(level) + 0.5) as f32;
        let rings = marching_squares::execute(
            |r, c| raster.get_scalar(Coord { r, c }),
            raster.rows(),
            raster.cols(),
            isoval,
            Coord {
                r: raster.gsize,
                c: raster.gsize,
            },
        );
        let deeper: Vec<Polygon> = rings
            .into_iter()
            .map(|ring| Polygon::new(ring.into_iter().map(|crd| raster.to_point(crd)).collect()))
            .collect();
        let deeper_ex = union_ex(&deeper, FillRule::NonZero)?;
        let inner = intersection_ex(&deeper_ex, &remaining)?;
        let outer = difference_ex(&remaining, &inner)?;
        shells.push(AdaptiveInfillShell {
            density: (f64::from(spec.surface_density) * levels.scale(level)) as f32,
            expolygons: outer,
        });
        remaining = inner;
    }
    if !remaining.is_empty() {
        shells.push(AdaptiveInfillShell {
            density: spec.interior_density,
            expolygons: remaining,
        });
    }
    Ok(shells)
}

/// `fill_adaptive_infill_shells` (`AdaptiveInfillField.cpp:811-821`).
pub(crate) fn fill_adaptive_infill_shells<F>(
    field: &AdaptiveInfillField,
    expolygon: &ExPolygon,
    spec: ShellFillSpec,
    scale: CoordinateScale,
    mut fill_shell: F,
) -> Result<(), ClipperError>
where
    F: FnMut(f32, &ExPolygon) -> Result<(), ClipperError>,
{
    let delta = -((0.5 * spec.spacing / scale.factor()) as f32);
    for shell in make_adaptive_infill_shells(field, expolygon, spec, scale)? {
        for part in offset_expolygons(&shell.expolygons, delta, JoinType::Miter, 3.0)? {
            fill_shell(shell.density, &part)?;
        }
    }
    Ok(())
}
