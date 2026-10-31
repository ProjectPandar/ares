//! libnest2d's `_Item<ExPolygon>` (`nester.hpp:46-470` at `345efec64`): a shape with its
//! inflation, rotation and translation, cached as upstream caches them.

use std::cell::OnceCell;

use super::geometry::{NBox, area, leftmost_down_vertex, offset, rightmost_up_vertex, shape_box};
use crate::geometry::{ExPolygon, Point, Polygon};

/// `BIN_ID_UNSET` / `BIN_ID_UNFIT`.
pub(crate) const BIN_ID_UNSET: i32 = -1;

#[derive(Clone, Debug)]
pub(crate) struct Item {
    shape: ExPolygon,
    translation: Point,
    rotation: f64,
    inflation: i64,
    has_rotation: bool,
    has_translation: bool,
    has_inflation: bool,
    inflated: OnceCell<ExPolygon>,
    transformed: OnceCell<ExPolygon>,
    /// The rotated inflated shape's box, without the translation.
    rotated_box: OnceCell<NBox>,
    area: OnceCell<f64>,
    pub(crate) bin_id: i32,
    pub(crate) priority: i32,
    pub(crate) fixed: bool,
    pub(crate) item_id: i32,
    pub(crate) extrude_ids: Vec<i32>,
    pub(crate) filament_temp_type: i32,
    pub(crate) height: f64,
    pub(crate) z_min: f64,
    pub(crate) z_max: f64,
    pub(crate) has_z_range: bool,
    pub(crate) is_bed_exclusion: bool,
    pub(crate) bed_exclusion_extruder_ids: Vec<i32>,
    pub(crate) bed_exclusion_extruder_id: i32,
    pub(crate) print_temp: f64,
    pub(crate) bed_temp: f64,
    pub(crate) vitrify_temp: f64,
    pub(crate) name: String,
    pub(crate) is_virt_object: bool,
    pub(crate) is_wipe_tower: bool,
    pub(crate) has_tried_with_excluded: bool,
}

impl Item {
    pub(crate) fn new(shape: ExPolygon) -> Self {
        Self {
            shape,
            translation: Point::new(0, 0),
            rotation: 0.0,
            inflation: 0,
            has_rotation: false,
            has_translation: false,
            has_inflation: false,
            inflated: OnceCell::new(),
            transformed: OnceCell::new(),
            rotated_box: OnceCell::new(),
            area: OnceCell::new(),
            bin_id: BIN_ID_UNSET,
            priority: 0,
            fixed: false,
            item_id: 0,
            extrude_ids: Vec::new(),
            filament_temp_type: -1,
            height: 0.0,
            z_min: 0.0,
            z_max: 0.0,
            has_z_range: false,
            is_bed_exclusion: false,
            bed_exclusion_extruder_ids: Vec::new(),
            bed_exclusion_extruder_id: -1,
            print_temp: 0.0,
            bed_temp: 0.0,
            vitrify_temp: 0.0,
            name: String::new(),
            is_virt_object: false,
            is_wipe_tower: false,
            has_tried_with_excluded: false,
        }
    }

    /// `markAsFixedInBin`.
    pub(crate) fn mark_as_fixed_in_bin(&mut self, bin_id: i32) {
        self.fixed = bin_id >= 0;
        self.bin_id = bin_id;
    }

    pub(crate) fn translation(&self) -> Point {
        self.translation
    }

    pub(crate) fn rotation(&self) -> f64 {
        self.rotation
    }

    /// `translation(tr)`: only a change marks the item translated.
    pub(crate) fn set_translation(&mut self, translation: Point) {
        if self.translation != translation {
            self.translation = translation;
            self.has_translation = true;
            self.transformed = OnceCell::new();
        }
    }

    pub(crate) fn translate(&mut self, by: Point) {
        self.set_translation(Point::new(
            self.translation.x() + by.x(),
            self.translation.y() + by.y(),
        ));
    }

    /// `rotation(rot)`: only a change marks the item rotated.
    pub(crate) fn set_rotation(&mut self, rotation: f64) {
        if self.rotation != rotation {
            self.rotation = rotation;
            self.has_rotation = true;
            self.transformed = OnceCell::new();
            self.rotated_box = OnceCell::new();
        }
    }

    pub(crate) fn rotate(&mut self, by: f64) {
        self.set_rotation(self.rotation + by);
    }

    /// `inflation(distance)`: marks the item inflated even at zero, which normalizes the shape
    /// through the offset.
    pub(crate) fn set_inflation(&mut self, distance: i64) {
        self.inflation = distance;
        self.has_inflation = true;
        self.inflated = OnceCell::new();
        self.transformed = OnceCell::new();
        self.rotated_box = OnceCell::new();
        self.area = OnceCell::new();
    }

    pub(crate) fn inflate(&mut self, distance: i64) {
        self.set_inflation(self.inflation + distance);
    }

    fn inflated_shape(&self) -> &ExPolygon {
        if !self.has_inflation {
            return &self.shape;
        }
        self.inflated
            .get_or_init(|| offset(&self.shape, self.inflation))
    }

    /// `transformedShape`: inflated, rotated, then translated.
    pub(crate) fn transformed_shape(&self) -> &ExPolygon {
        self.transformed.get_or_init(|| {
            let mut shape = self.inflated_shape().clone();
            if self.has_rotation {
                shape = rotated(&shape, self.rotation);
            }
            if self.has_translation {
                shape = translated(&shape, self.translation);
            }
            shape
        })
    }

    /// `area`: of the inflated shape.
    pub(crate) fn area(&self) -> f64 {
        *self.area.get_or_init(|| area(self.inflated_shape()))
    }

    /// `boundingBox`: the rotated inflated shape's envelope moved by the translation.
    pub(crate) fn bounding_box(&self) -> NBox {
        let rotated_box = *self.rotated_box.get_or_init(|| {
            if self.has_rotation {
                shape_box(&rotated(self.inflated_shape(), self.rotation))
            } else {
                shape_box(self.inflated_shape())
            }
        });
        NBox::new(
            Point::new(
                rotated_box.min.x() + self.translation.x(),
                rotated_box.min.y() + self.translation.y(),
            ),
            Point::new(
                rotated_box.max.x() + self.translation.x(),
                rotated_box.max.y() + self.translation.y(),
            ),
        )
    }

    /// `rightmostTopVertex`, the reference vertex.
    pub(crate) fn reference_vertex(&self) -> Point {
        rightmost_up_vertex(self.transformed_shape().contour().points())
    }

    pub(crate) fn leftmost_bottom_vertex(&self) -> Point {
        leftmost_down_vertex(self.transformed_shape().contour().points())
    }
}

/// `ExPolygon::rotate(angle)` (`MultiPoint::rotate`, rounding each vertex).
pub(crate) fn rotated(shape: &ExPolygon, angle: f64) -> ExPolygon {
    let (sin, cos) = angle.sin_cos();
    let turn = |polygon: &Polygon| {
        let mut polygon = polygon.clone();
        polygon.rotate(cos, sin);
        polygon
    };
    ExPolygon::new(
        turn(shape.contour()),
        shape.holes().iter().map(turn).collect(),
    )
}

pub(crate) fn translated(shape: &ExPolygon, by: Point) -> ExPolygon {
    let shift = |polygon: &Polygon| {
        Polygon::new(
            polygon
                .points()
                .iter()
                .map(|point| Point::new(point.x() + by.x(), point.y() + by.y()))
                .collect(),
        )
    };
    ExPolygon::new(
        shift(shape.contour()),
        shape.holes().iter().map(shift).collect(),
    )
}
