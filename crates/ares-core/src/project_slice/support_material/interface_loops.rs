//! Geometry of `SupportCommon.cpp::LoopInterfaceProcessor:831-1058`.
//! The public configuration permits zero or one contact loop.

mod centers;
pub(super) mod extrude;
mod stitch;

use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Point, Polygon, Polyline,
    difference_polygons_paths, intersection_open_polylines, offset_expolygons,
    offset_expolygons_paths, offset_open_paths, union_ex,
};

pub(super) struct LoopInterfaceProcessor {
    enabled: bool,
    radius: f64,
    distance: f64,
    circle: Polygon,
}

pub(super) struct LoopGeometry {
    pub(super) lines: Vec<Polyline>,
    pub(super) remaining: Vec<Polygon>,
}

impl LoopInterfaceProcessor {
    pub(super) fn new(enabled: bool, radius: f64) -> Self {
        let circle = Polygon::new(
            (0..6)
                .map(|index| {
                    let angle = index as f64 * std::f64::consts::PI / 3.0;
                    Point::new(
                        (radius * angle.cos()).round() as i64,
                        (radius * angle.sin()).round() as i64,
                    )
                })
                .collect(),
        );
        Self {
            enabled,
            radius,
            distance: radius * 3.0,
            circle,
        }
    }

    pub(super) fn generate(
        &self,
        polygons: &[Polygon],
        overhang_polygons: Option<&mut Vec<Polygon>>,
        // Flow::scaled_width after with_height; not float scale_(width).
        flow_width_scaled: i64,
        scale: CoordinateScale,
    ) -> Result<Option<LoopGeometry>, ClipperError> {
        if !self.enabled || polygons.is_empty() {
            return Ok(None);
        }
        let overhangs = overhang_polygons.map(std::mem::take).unwrap_or_default();
        let half_width = 0.5_f32 * flow_width_scaled as f32;
        let contacts = offset_expolygons(
            &union_ex(polygons, FillRule::NonZero)?,
            -half_width,
            JoinType::Miter,
            3.0,
        )?;
        let margin = offset_expolygons_paths(
            &union_ex(&overhangs, FillRule::NonZero)?,
            half_width,
            JoinType::Miter,
            3.0,
        )?;
        let mut external = Vec::new();
        let mut circles = Vec::new();
        for contact in contacts {
            place_contact_loops(self, contact, &margin, scale, (&mut external, &mut circles))?;
        }
        let loops = difference_polygons_paths(&external, &circles)?;
        let lines = stitch::clip_and_stitch(&loops, &overhangs, scale)?;
        let paths = lines
            .iter()
            .cloned()
            .map(Polyline::into_points)
            .map(Polygon::new)
            .collect::<Vec<_>>();
        let covered = offset_open_paths(&paths, (self.radius * 1.1) as f32, JoinType::Square, 0.0)?;
        Ok(Some(LoopGeometry {
            lines,
            remaining: difference_polygons_paths(polygons, &covered)?,
        }))
    }
}

fn place_contact_loops(
    processor: &LoopInterfaceProcessor,
    contact: ExPolygon,
    margin: &[Polygon],
    scale: CoordinateScale,
    (external, circles): (&mut Vec<Polygon>, &mut Vec<Polygon>),
) -> Result<(), ClipperError> {
    let mut placement = centers::CircleCenters::new(processor.distance, scale);
    let (contour, holes) = contact.into_parts();
    for contour in std::iter::once(contour).chain(holes) {
        if intersection_open_polylines(&[contour.split_at_first_point()], margin)?.is_empty() {
            continue;
        }
        placement.walk(&contour);
        external.push(contour);
        for center in placement.points() {
            circles.push(Polygon::new(
                processor
                    .circle
                    .points()
                    .iter()
                    .map(|point| Point::new(point.x() + center.x(), point.y() + center.y()))
                    .collect(),
            ));
        }
    }
    Ok(())
}
