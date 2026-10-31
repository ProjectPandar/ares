//! The plate's prime tower (`PartPlate.cpp` at v2.5.0-beta `345efec64`): its footprint
//! (`estimate_wipe_tower_footprint`, `:3982-4034`), the objects it keeps clear of
//! (`wipe_tower_obstacles`, `:4036-4063`), its arrange item at the stored position
//! (`estimate_wipe_tower_polygon`, `:4065-4155`) and the default position of a new plate
//! (`tower_default.rs`).

use super::PlateList;
use crate::arrangement::ArrangePolygon;
use crate::geometry::{ExPolygon, JoinType, Point, Polygon, offset_paths};
use crate::options::ProcessBrimType;
use crate::plater::arrange_job::convex_hull_2d;
use crate::plater::selection::instance_bounding_box;
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::{DynamicConfig, get_at};
use crate::project_slice::gcode_emit::wipe_tower_estimate::{
    Footprint, estimate_footprint, first_layer_outline, nearest_clear_anchor,
    resolve_wipe_tower_type,
};
use crate::{Project, SliceError};

/// `WIPE_TOWER_MARGIN` and `WIPE_TOWER_AUTO_MARGIN` (`libslic3r.h:100-103`).
const WIPE_TOWER_MARGIN: f64 = 1.0;
pub(super) const WIPE_TOWER_AUTO_MARGIN: f64 = 15.0;

/// `scaled(float)`: divided in float, truncated.
pub(super) fn scaled_f32(value: f32) -> i64 {
    (value / 1e-6_f32) as i64
}

/// `scaled(double)`.
pub(super) fn scaled(value: f64) -> i64 {
    (value / 1e-6) as i64
}

fn unscaled(value: i64) -> f64 {
    value as f64 * 1e-6
}

/// `std::clamp`.
fn clamp(value: f32, lo: f32, hi: f32) -> f32 {
    if value < lo {
        lo
    } else if hi < value {
        hi
    } else {
        value
    }
}

/// `WipeTowerRelocation`: what the estimate did to a stored position that no longer fits.
#[derive(Clone, Copy, Debug, Default)]
pub(in crate::plater) struct Relocation {
    pub(in crate::plater) moved: bool,
    pub(in crate::plater) overlaps_objects: bool,
}

/// The tower estimate's inputs: the plates, the model, the full config and the stored tower
/// positions (`wipe_tower_x`, `wipe_tower_y`).
pub(in crate::plater) struct Towers<'a> {
    pub(in crate::plater) plates: &'a PlateList,
    pub(in crate::plater) project: &'a Project,
    pub(in crate::plater) config: &'a DynamicConfig,
    pub(in crate::plater) positions: (&'a [f64], &'a [f64]),
}

impl Towers<'_> {
    /// `estimate_wipe_tower_footprint(config, plate_extruder_size, use_global_objects)` of the
    /// plate at `index`.
    pub(in crate::plater) fn footprint(
        &self,
        index: usize,
        plate_extruder_size: usize,
        use_global_objects: bool,
    ) -> Result<Footprint, SliceError> {
        let (plates, project, config) = (self.plates, self.project, self.config);
        let mut extruders = plates.plate_extruders(index, project, config)?;
        let mut id = 1;
        while extruders.len() < plate_extruder_size {
            if !extruders.contains(&id) {
                extruders.push(id);
            }
            id += 1;
        }
        let tower_filament = config.option("wipe_tower_filament").int() as i32;
        if config.option("enable_prime_tower").bool()
            && extruders.len() > 1
            && tower_filament > 0
            && !extruders.contains(&tower_filament)
        {
            extruders.push(tower_filament);
        }
        if extruders.is_empty() {
            return Ok(Footprint::default());
        }
        let global_layer_height = config.option("layer_height").float();
        let (mut max_height, mut layer_height) = (0.0_f64, f64::MAX);
        for (object_index, object) in project.objects().iter().enumerate() {
            if !use_global_objects && !plates.contains_any_totally(index, object_index, project)? {
                continue;
            }
            max_height =
                max_height.max(self.counted_height(index, object_index, use_global_objects)?);
            let own = object
                .object_overrides()
                .layer_height
                .map(|height| height.0);
            layer_height = layer_height.min(own.unwrap_or(global_layer_height));
        }
        if layer_height == f64::MAX {
            layer_height = global_layer_height;
        }
        let filament_ids: Vec<u32> = extruders
            .iter()
            .filter(|&&id| id > 0)
            .map(|&id| (id - 1) as u32)
            .collect();
        Ok(estimate_footprint(
            config,
            resolve_wipe_tower_type(config),
            &filament_ids,
            layer_height,
            max_height,
        ))
    }

    /// The tallest instance of the object the footprint counts (`instance_convex_hull_bounding_box`
    /// per instance, `:4011-4016`).
    fn counted_height(
        &self,
        index: usize,
        object_index: usize,
        use_global_objects: bool,
    ) -> Result<f64, SliceError> {
        let mut height = 0.0_f64;
        for instance in 0..self.project.objects()[object_index].instances().len() {
            let key = (object_index, instance);
            if use_global_objects || self.plates.contains_totally(index, key, self.project)? {
                let (min, max) = instance_bounding_box(self.project, key);
                height = height.max(max[2] - min[2]);
            }
        }
        Ok(height)
    }

    /// The footprint the relocation keeps clear, turned as the tower is (`:4104-4111`).
    fn tower_outline(&self, (brim, w, depth): (f32, f32, f32)) -> Polygon {
        let mut tower = Polygon::new(vec![
            Point::new(scaled_f32(-brim), scaled_f32(-brim)),
            Point::new(scaled_f32(w + brim), scaled_f32(-brim)),
            Point::new(scaled_f32(w + brim), scaled_f32(depth + brim)),
            Point::new(scaled_f32(-brim), scaled_f32(depth + brim)),
        ]);
        let rotation = self.config.option("wipe_tower_rotation_angle").float();
        if rotation != 0.0 {
            let (sin, cos) = rotation.to_radians().sin_cos();
            tower.rotate(cos, sin);
        }
        tower
    }

    /// The relocation of a re-placed tower (`:4112-4131`): `None` where the re-placed spot is
    /// clear, else the clear spot nearest the saved one in the comfort, then the valid area, or
    /// `Some(None)` where none is clear.
    fn relocate(
        &self,
        index: usize,
        tower: &Polygon,
        (placed, saved): (Point, Point),
        areas: [(Point, Point); 2],
    ) -> Option<Option<Point>> {
        let objects = self.obstacles(index);
        if nearest_clear_anchor((placed, placed), tower, &objects, placed).is_some() {
            return None;
        }
        let [comfort, valid] = areas;
        Some(
            nearest_clear_anchor(comfort, tower, &objects, saved)
                .or_else(|| nearest_clear_anchor(valid, tower, &objects, saved)),
        )
    }

    /// `wipe_tower_obstacles`: each printable instance's outline with its brim and the margin, in
    /// the plate's frame.
    fn obstacles(&self, index: usize) -> Vec<Polygon> {
        let plate = &self.plates.plates[index];
        let mut obstacles = Vec::new();
        for &(object_index, instance) in &plate.instances {
            let object = &self.project.objects()[object_index];
            let placed = &object.instances()[instance];
            if !placed.printable() {
                continue;
            }
            let hull = convex_hull_2d(object, placed.transform());
            if hull.points().len() < 3 {
                continue;
            }
            let (dx, dy) = (scaled(plate.origin[0]), scaled(plate.origin[1]));
            let hull = Polygon::new(
                hull.points()
                    .iter()
                    .map(|p| Point::new(p.x() - dx, p.y() - dy))
                    .collect(),
            );
            let options = object.object_overrides();
            let has_brim = match options.brim_type {
                Some(brim_type) => brim_type != ProcessBrimType::NoBrim,
                None => self.config.option("brim_type").key() != "no_brim",
            };
            let brim_width = options
                .brim_width
                .map_or_else(|| self.config.option("brim_width").float(), |width| width.0);
            let brim = if has_brim { brim_width } else { 0.0 };
            let delta = scaled(brim + WIPE_TOWER_MARGIN) as f32;
            obstacles.extend(
                offset_paths(&[hull], delta, JoinType::Miter, 3.0).expect("obstacle offset"),
            );
        }
        obstacles
    }

    /// `estimate_wipe_tower_polygon` of the plate at `index` for the bed `plate_index`: the
    /// stored position kept, snapped onto the plate or re-placed, and the tower item there.
    pub(in crate::plater) fn polygon(
        &self,
        (index, plate_index): (usize, usize),
        plate_extruder_size: usize,
        relocation: Option<&mut Relocation>,
    ) -> Result<(ArrangePolygon, [f64; 2], Footprint), SliceError> {
        let mut x = get_at(self.positions.0, plate_index) as f32;
        let mut y = get_at(self.positions.1, plate_index) as f32;
        let footprint = self.footprint(index, plate_extruder_size, false)?;
        let (plate_width, plate_depth) = self.plates.size;
        let (w, depth) = (footprint.width as f32, footprint.depth as f32);
        let mut brim = footprint.brim_width as f32;
        let outline = first_layer_outline(
            self.config,
            resolve_wipe_tower_type(self.config),
            (f64::from(w), f64::from(depth), footprint.height),
        );
        let points = outline.points();
        let max_x = points.iter().map(|p| p.x()).max().expect("outline");
        let max_y = points.iter().map(|p| p.y()).max().expect("outline");
        let min_x = points.iter().map(|p| p.x()).min().expect("outline");
        let min_y = points.iter().map(|p| p.y()).min().expect("outline");
        let bulge = [
            unscaled(max_x) - f64::from(w),
            unscaled(max_y) - f64::from(depth),
            -unscaled(min_x),
            -unscaled(min_y),
        ]
        .into_iter()
        .fold(0.0_f64, f64::max);
        brim += bulge as f32;
        let margin = (WIPE_TOWER_MARGIN + f64::from(brim)) as f32;
        let x_hi = margin.max(plate_width as f32 - w - margin);
        let y_hi = margin.max(plate_depth as f32 - depth - margin);
        let margin_c = WIPE_TOWER_AUTO_MARGIN as f32 + brim;
        let (mut x_lo_c, mut x_hi_c) = (margin_c, plate_width as f32 - w - margin_c);
        if x_lo_c > x_hi_c {
            (x_lo_c, x_hi_c) = (margin, x_hi);
        }
        let (mut y_lo_c, mut y_hi_c) = (margin_c, plate_depth as f32 - depth - margin_c);
        if y_lo_c > y_hi_c {
            (y_lo_c, y_hi_c) = (margin, y_hi);
        }
        let tol = 5.0_f32;
        let (saved_x, saved_y) = (x, y);
        let replace_x = x < margin - tol || x > x_hi + tol;
        let replace_y = y < margin - tol || y > y_hi + tol;
        x = if replace_x {
            clamp(x, x_lo_c, x_hi_c)
        } else {
            clamp(x, margin, x_hi)
        };
        y = if replace_y {
            clamp(y, y_lo_c, y_hi_c)
        } else {
            clamp(y, margin, y_hi)
        };
        if let Some(relocation) = relocation.filter(|_| replace_x || replace_y) {
            relocation.moved = true;
            let tower = self.tower_outline((brim, w, depth));
            let placed = Point::new(scaled_f32(x), scaled_f32(y));
            let saved = Point::new(scaled_f32(saved_x), scaled_f32(saved_y));
            let comfort = (
                Point::new(scaled_f32(x_lo_c), scaled_f32(y_lo_c)),
                Point::new(scaled_f32(x_hi_c), scaled_f32(y_hi_c)),
            );
            let valid = (
                Point::new(scaled_f32(margin), scaled_f32(margin)),
                Point::new(scaled_f32(x_hi), scaled_f32(y_hi)),
            );
            match self.relocate(index, &tower, (placed, saved), [comfort, valid]) {
                None => {}
                Some(Some(clear)) => {
                    (x, y) = (unscaled(clear.x()) as f32, unscaled(clear.y()) as f32)
                }
                Some(None) => relocation.overlaps_objects = true,
            }
        }
        let contour = Polygon::new(vec![
            Point::new(scaled_f32(x - brim), scaled_f32(y - brim)),
            Point::new(scaled_f32(x + w + brim), scaled_f32(y - brim)),
            Point::new(scaled_f32(x + w + brim), scaled_f32(y + depth + brim)),
            Point::new(scaled_f32(x - brim), scaled_f32(y + depth + brim)),
        ]);
        let ap = ArrangePolygon {
            poly: ExPolygon::new(contour, Vec::new()),
            bed_idx: plate_index as i32,
            name: "WipeTower".to_owned(),
            is_virt_object: true,
            is_wipe_tower: true,
            ..ArrangePolygon::default()
        };
        Ok((ap, [f64::from(x), f64::from(y)], footprint))
    }
}
