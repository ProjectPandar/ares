//! `TreeSupport.cpp:1291-1353::make_perimeter_and_infill`.

use super::{
    fill::{FillSettings, Filler},
    loops::make_loops,
};
use crate::{
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon,
        chain_points, difference_ex, offset_expolygon,
    },
    project_slice::{
        perimeters::{classic::shortest_path::ChainEntity, types::Flow},
        support_material::SupportExtrusionEntity,
    },
};

pub(super) struct PerimeterInfill {
    pub(super) wall_count: usize,
    pub(super) fill: FillSettings,
    pub(super) infill_first: bool,
}

impl Filler {
    pub(super) fn perimeter_and_infill(
        &self,
        destination: &mut Vec<SupportExtrusionEntity>,
        area: &ExPolygon,
        settings: PerimeterInfill,
    ) -> Result<(), ClipperError> {
        let scale = self.scale;
        let spacing = settings.fill.flow.scaled_spacing(scale) as f32;
        let mut outlines = offset_expolygon(area, -0.5 * spacing, JoinType::Square, 3.0)?;
        let infill = offset_expolygon(
            area,
            -(settings.wall_count as f32) * spacing,
            JoinType::Square,
            3.0,
        )?;
        let boxes = self.append(destination, &infill, settings.fill)?;
        if settings.wall_count == 0 {
            outlines = trim_zero_wall(outlines, boxes, self.angle, settings.fill.flow, scale)?;
        }
        let mut loops = make_loops(
            outlines,
            settings.wall_count,
            settings.fill.flow,
            settings.fill.role,
            scale,
        )?;
        if settings.infill_first {
            destination.append(&mut loops);
        } else {
            loops.append(destination);
            *destination = loops;
        }
        destination.retain(|entity| !matches!(entity, SupportExtrusionEntity::NoSort(children) if children.is_empty()));
        if settings.infill_first {
            let points = destination
                .iter()
                .map(ChainEntity::first_point)
                .collect::<Vec<_>>();
            let mut entities = std::mem::take(destination)
                .into_iter()
                .map(Some)
                .collect::<Vec<_>>();
            destination.extend(
                chain_points(&points)
                    .into_iter()
                    .map(|index| entities[index].take().unwrap()),
            );
        }
        Ok(())
    }
}

fn trim_zero_wall(
    mut outlines: Vec<ExPolygon>,
    boxes: Vec<BoundingBox>,
    angle: f32,
    flow: Flow,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let width = flow.scaled_width(scale);
    for bounds in boxes {
        let (mut min, mut max) = (bounds.min(), bounds.max());
        let extension = scale.scaled_delta(10.0) as i64;
        if f64::from(angle).cos() >= f64::from(angle).sin() {
            min = Point::new(min.x() - extension, min.y());
            max = Point::new(max.x() + extension, max.y());
        } else {
            min = Point::new(min.x(), min.y() - extension);
            max = Point::new(max.x(), max.y() + extension);
        }
        let box_area = ExPolygon::new(
            Polygon::new(vec![
                min,
                Point::new(max.x(), min.y()),
                max,
                Point::new(min.x(), max.y()),
            ]),
            Vec::new(),
        );
        let mask = offset_expolygon(&box_area, (0.5 * width as f64) as f32, JoinType::Miter, 3.0)?;
        outlines = difference_ex(&outlines, &mask)?;
    }
    let mut retained = Vec::with_capacity(outlines.len());
    for outline in outlines {
        if !offset_expolygon(&outline, -width as f32, JoinType::Miter, 3.0)?.is_empty() {
            retained.push(outline);
        }
    }
    Ok(retained)
}
