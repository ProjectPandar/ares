//! `GCodeProcessor.cpp:1866-1999`: non-custom extrusion-group bounds.
use std::collections::BTreeMap;

use crate::geometry::{BoundingBox, CoordinateScale, Point, Polygon, difference_polygons_paths};
use crate::options::Nullable;
use crate::{Point2d, ProjectSettings, SliceError};

pub(crate) struct PrintableAreaTracker {
    groups: BTreeMap<(i32, usize), Group>,
    object: i32,
    print_z: f32,
    custom: bool,
    offsets: Vec<[f32; 2]>,
    xy_offset: [f64; 2],
}

struct Group {
    bounds: BoundingBox,
    max_print_z: f32,
}

impl PrintableAreaTracker {
    pub(crate) fn new(settings: &ProjectSettings, xy_offset: [f64; 2]) -> Self {
        let map = &settings.project.gcode.filament_map.0;
        let offsets = &settings.project.print.extruder_offset.0;
        let offsets = (0..settings.filament.gcode.filament_diameter.0.len())
            .map(|filament| {
                let slot = map
                    .get(filament)
                    .map_or(0, |slot| (slot.0 - 1).max(0) as usize);
                offsets
                    .get(slot)
                    .or(offsets.first())
                    .map_or([0.0; 2], |p| [p.x as f32, p.y as f32])
            })
            .collect();
        Self {
            groups: BTreeMap::new(),
            object: -1,
            print_z: 0.0,
            custom: false,
            offsets,
            xy_offset,
        }
    }

    /// `process_tags`, :3121-3143; unknown/malformed numeric tags retain Native defaults.
    pub(crate) fn tag(&mut self, line: &str, bbl: bool) {
        let Some(comment) = line.strip_prefix(';') else {
            return;
        };
        let role = if bbl { " FEATURE: " } else { "TYPE:" };
        if let Some(role) = comment.strip_prefix(role) {
            self.custom = role == "Custom";
        } else if comment.starts_with(" start printing object") {
            self.object = comment
                .split_once(':')
                .and_then(|(_, v)| v.split_whitespace().next())
                .and_then(|v| v.parse().ok())
                .unwrap_or(-1);
        } else if comment.starts_with(" stop printing object") {
            self.object = -1;
        } else if let Some(z) = comment.strip_prefix(" Z_HEIGHT:") {
            self.print_z = z
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);
        }
    }

    pub(crate) fn motion(&mut self, delta: [f64; 4], end: [f64; 4], wiping: bool, filament: usize) {
        if wiping || self.custom || delta[3] <= 0.0 || delta[0] == 0.0 && delta[1] == 0.0 {
            return;
        }
        // `store_move_vertex:5680`: addition is in f32 before conversion to fixed XY.
        let offset = self.offsets.get(filament).copied().unwrap_or_default();
        let p = Point::new(
            scale(f64::from((end[0] + self.xy_offset[0]) as f32 + offset[0])),
            scale(f64::from((end[1] + self.xy_offset[1]) as f32 + offset[1])),
        );
        let group = self.groups.entry((self.object, filament)).or_insert(Group {
            bounds: BoundingBox::new(p, p),
            max_print_z: 0.0,
        });
        group.bounds.merge_point(p);
        group.max_print_z = group.max_print_z.max(self.print_z);
    }

    /// Called by the core `GCode::export` completion boundary, not standalone move decoding.
    pub(crate) fn check(&self, settings: &ProjectSettings) -> Result<u8, SliceError> {
        let printer = &settings.printer.remaining;
        let plate = polygon(&printer.printable_area.0);
        let offset = Point::new(scale(self.xy_offset[0]), scale(self.xy_offset[1]));
        let plate_bbox = (plate.points().len() >= 3)
            .then(|| BoundingBox::from_polygon(&plate))
            .flatten()
            .map(|b| translated(b, offset).inflated(scale(2.0)));
        let multiple = settings.project.print.nozzle_diameter.0.len() > 1;
        let exclusions = if multiple {
            exclusion_bboxes(&plate, &printer.extruder_printable_area.0, offset)?
        } else {
            Vec::new()
        };
        let mut error_code = 0;
        for ((_, filament), group) in &self.groups {
            if plate_bbox.is_some_and(|plate| !contains(plate, group.bounds)) {
                error_code |= 1 << 2;
            }
            if f64::from(group.max_print_z) > printer.printable_height.0 {
                error_code |= 1 << 3;
            }
            if !multiple {
                continue;
            }
            let extruder = settings.project.gcode.filament_map.0[*filament].0 as usize - 1;
            if exclusions
                .get(extruder)
                .is_some_and(|areas| areas.iter().any(|area| overlaps(*area, group.bounds)))
            {
                error_code |= 1;
            }
            if let Some(Nullable::Value(height)) = printer.extruder_printable_height.0.get(extruder)
                && f64::from(group.max_print_z) > height.0
            {
                error_code |= 1 << 1;
            }
        }
        Ok(error_code)
    }
}

// Print.cpp:3267-3278: difference contours, then bboxes (not point-in-polygon tests).
fn exclusion_bboxes(
    plate: &Polygon,
    areas: &[Vec<Point2d>],
    offset: Point,
) -> Result<Vec<Vec<BoundingBox>>, SliceError> {
    areas
        .iter()
        .map(|points| {
            let paths = difference_polygons_paths(std::slice::from_ref(plate), &[polygon(points)])
                .map_err(|error| {
                    SliceError::InvalidInput(format!(
                        "extruder printable-area difference failed: {error:?}"
                    ))
                })?;
            Ok(paths
                .iter()
                .filter_map(BoundingBox::from_polygon)
                .map(|b| translated(b, offset))
                .collect())
        })
        .collect()
}

fn scale(v: f64) -> i64 {
    CoordinateScale::Normal.scaled_delta(v) as i64
}

fn polygon(points: &[Point2d]) -> Polygon {
    Polygon::new(
        points
            .iter()
            .map(|p| Point::new(scale(p.x), scale(p.y)))
            .collect(),
    )
}

fn translated(b: BoundingBox, offset: Point) -> BoundingBox {
    BoundingBox::new(
        Point::new(b.min().x() + offset.x(), b.min().y() + offset.y()),
        Point::new(b.max().x() + offset.x(), b.max().y() + offset.y()),
    )
}

fn contains(outer: BoundingBox, inner: BoundingBox) -> bool {
    outer.min().x() <= inner.min().x()
        && outer.min().y() <= inner.min().y()
        && outer.max().x() >= inner.max().x()
        && outer.max().y() >= inner.max().y()
}

fn overlaps(a: BoundingBox, b: BoundingBox) -> bool {
    a.max().x() >= b.min().x()
        && a.min().x() <= b.max().x()
        && a.max().y() >= b.min().y()
        && a.min().y() <= b.max().y()
}
