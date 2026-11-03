//! "Scale to build volume" (`Plater::priv::scale_selection_to_fit_print_volume`,
//! `Plater.cpp:11092-11112` at v2.5.0-beta `345efec64`; `Selection::scale_to_fit_print_volume`,
//! `Selection.cpp:1481-1580`). See `docs/architecture/web-plater-scale-to-fit.md`.

use super::PlaterSession;
use super::plate_list::{BoundingBox, InstanceKey};
use super::selection::{self, TransformationType};
use crate::SliceError;
use crate::arrangement::SCALING_FACTOR;
use crate::geometry::enclosing_circle::smallest_enclosing_circle_welzl;
use crate::geometry::{Point, convex_hull};
use crate::project::ProjectVolumeType;
use crate::project_slice::{BedShape, Circle, PrintVolume};

/// `BuildVolume::SceneEpsilon` (`BuildVolume.hpp:110`).
const SCENE_EPSILON: f64 = 1e-4;

/// An exclusion box's X and Y extent: `(min, max)`.
type ExcludeBox = ([f64; 2], [f64; 2]);

impl PlaterSession {
    /// Fits one full instance into the current plate's free build volume.
    pub fn scale_to_fit(&mut self, object: usize, instance: usize) -> Result<(), SliceError> {
        self.check(object, instance)?;
        // `this->bed.build_volume()` (`Plater.cpp:11095`): the bed at the first plate.
        match PrintVolume::at(&self.project, (0.0, 0.0))?.shape() {
            BedShape::Rectangle { .. } => {}
            BedShape::Circle(circle) => return self.fit_circle((object, instance), circle),
            BedShape::Convex { .. } | BedShape::Unported => return Ok(()),
        }
        let printer = &self.project.settings().printer.remaining;
        let bed_box = self
            .plates
            .shared_build_volume(self.current_plate, &printer.extruder_printable_area.0)?;
        let origin = self.plates.plates[self.current_plate].origin;
        let excludes = exclude_boxes(&printer.bed_exclude_area.0, origin);
        let print_volumes = if excludes.is_empty() {
            vec![bed_box]
        } else {
            free_print_volumes(&bed_box, &excludes)
        };
        self.fit_rectangle((object, instance), &print_volumes);
        Ok(())
    }

    /// `fit_rectangle` (`Selection.cpp:1511-1532`): the volume giving the strictly largest
    /// `min(sx, sy, sz)`, the first of equals.
    fn fit_rectangle(&mut self, key: InstanceKey, print_volumes: &[BoundingBox]) {
        let bbox = selection::selection_bounding_box(&self.project, key);
        let box_size: [f64; 3] = std::array::from_fn(|axis| bbox.1[axis] - bbox.0[axis] + 0.02);
        let mut best_s = 0.0;
        let mut best = None;
        for print_volume in print_volumes {
            let ratios: [f64; 3] = std::array::from_fn(|axis| {
                ratio(print_volume.1[axis] - print_volume.0[axis], box_size[axis])
            });
            let s = ratios[0].min(ratios[1].min(ratios[2]));
            if ratios.iter().all(|&ratio| ratio != 0.0) && s > best_s {
                best_s = s;
                best = Some(print_volume);
            }
        }
        if let Some(best) = best {
            let offset = std::array::from_fn(|axis| {
                0.5 * (best.0[axis] + best.1[axis]) - 0.5 * (bbox.0[axis] + bbox.1[axis])
            });
            self.fit(key, best_s, offset);
        }
    }

    /// `fit_circle` (`Selection.cpp:1534-1569`): the selection's 2D hull points' enclosing circle and
    /// hull height into the bed circle (`BuildVolume::circle()`, scaled) and printable height,
    /// centred on the current plate's bed circle.
    fn fit_circle(&mut self, key: InstanceKey, print_circle: &Circle) -> Result<(), SliceError> {
        let print_circle_radius = unscale(print_circle.radius);
        if print_circle_radius == 0.0 {
            return Ok(());
        }
        let object = &self.project.objects()[key.0];
        // A non-model-part volume has no `GLVolume` convex hull (`3DScene.cpp:921-927`), which
        // upstream dereferences: undefined behaviour, refused.
        if object
            .volumes()
            .iter()
            .any(|volume| volume.volume_type() != ProjectVolumeType::ModelPart)
        {
            return Err(SliceError::InvalidInput(
                "scale to build volume on a circular bed needs every volume to be a model part"
                    .to_owned(),
            ));
        }
        let instance = object.instances()[key.1].transform();
        let mut points = Vec::new();
        let mut max_z: f64 = 0.0;
        for volume in object.volumes() {
            // The hull mesh through the world matrix, its vertices stored as floats
            // (`TriangleMesh::transform`); the 3D hull keeps every extreme vertex of the mesh.
            let world = instance.then(volume.transform());
            let vertices: Vec<[f64; 3]> = volume
                .mesh()
                .vertices()
                .iter()
                .map(|vertex| {
                    let p = world.transform_point(*vertex);
                    [p.x, p.y, p.z].map(|value| f64::from(value as f32))
                })
                .collect();
            let (min_z, top_z) = vertices
                .iter()
                .fold((f64::MAX, f64::MIN), |(low, high), v| {
                    (low.min(v[2]), high.max(v[2]))
                });
            max_z = max_z.max(top_z - min_z);
            // `TriangleMesh::convex_hull` (`TriangleMesh.cpp:436-445`): `Point::new_scale`, truncating.
            let projected: Vec<Point> = vertices
                .iter()
                .map(|v| Point::new(scaled(v[0]), scaled(v[1])))
                .collect();
            points.extend(convex_hull(&projected));
        }
        if points.is_empty() {
            return Ok(());
        }
        let circle = smallest_enclosing_circle_welzl(&points);
        // 1/100 mm on all sides against false out of print volume detections (`:1554-1555`).
        let circle_radius = unscale(circle.radius) + 0.01;
        if circle_radius == 0.0 || max_z == 0.0 {
            return Ok(());
        }
        let printer = &self.project.settings().printer.remaining;
        let printable_height = printer.printable_height.0;
        let s = (print_circle_radius / circle_radius).min(printable_height / max_z);
        let bbox = selection::selection_bounding_box(&self.project, key);
        let sel_center: [f64; 3] = std::array::from_fn(|axis| 0.5 * (bbox.0[axis] + bbox.1[axis]));
        let circle_center = [
            unscale(circle.center[0]),
            unscale(circle.center[1]),
            0.5 * max_z,
        ];
        let offset: [f64; 3] =
            std::array::from_fn(|axis| s * (circle_center[axis] - sel_center[axis]));
        // The plate's bed is the same circle moved by its centre (`get_center_origin`) minus
        // `BuildVolume::bed_center()`, the printable area's box centre.
        let bed_center = bed_center(&printer.printable_area.0);
        let plate_center = self.plates.center_origin(self.current_plate);
        let plate_shift = [
            plate_center[0] - bed_center[0],
            plate_center[1] - bed_center[1],
            0.0,
        ];
        let print_center = [
            unscale(print_circle.center[0]),
            unscale(print_circle.center[1]),
            0.5 * printable_height,
        ];
        let translation = std::array::from_fn(|axis| {
            (print_center[axis] + plate_shift[axis]) - (sel_center[axis] + offset[axis])
        });
        self.fit(key, s, translation);
        Ok(())
    }

    /// `fit` (`Selection.cpp:1483-1509`): a world, relative, joint scale and `do_scale`, then a
    /// relative world translate by `offset` with Z replaced by the scaled selection's lift onto
    /// the bed, and `do_move`.
    fn fit(&mut self, key: InstanceKey, s: f64, mut offset: [f64; 3]) {
        if s <= 0.0 || s == 1.0 {
            return;
        }
        self.scale_and_drop(key, ([s; 3], TransformationType::Relative));
        offset[2] = -selection::selection_bounding_box(&self.project, key).0[2];
        self.translate_instance(key.0, key.1, offset)
            .expect("the fitted instance exists");
    }
}

/// `PartPlate::get_exclude_areas` (`m_exclude_bounding_box`, `PartPlate.cpp:626-643`): every four
/// `bed_exclude_area` points moved by the plate origin (`set_shape`, `:5159-5161`), as a box; only
/// its X and Y are read.
fn exclude_boxes(area: &[crate::Point2d], origin: [f64; 2]) -> Vec<ExcludeBox> {
    area.as_chunks::<4>()
        .0
        .iter()
        .map(|corners| {
            let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
            for corner in corners {
                for (axis, value) in [corner.x + origin[0], corner.y + origin[1]]
                    .into_iter()
                    .enumerate()
                {
                    min[axis] = min[axis].min(value);
                    max[axis] = max[axis].max(value);
                }
            }
            (min, max)
        })
        .collect()
}

/// `free_print_volumes` (`Plater.cpp:11049-11090`): the boxes within `bed` spanned by bed and
/// exclusion edges that overlap no exclusion, without slivers of at most `2 * SceneEpsilon`.
fn free_print_volumes(bed: &BoundingBox, excludes: &[ExcludeBox]) -> Vec<BoundingBox> {
    let mut xs = vec![bed.0[0], bed.1[0]];
    let mut ys = vec![bed.0[1], bed.1[1]];
    for ex in excludes {
        for x in [ex.0[0], ex.1[0]] {
            if x > bed.0[0] && x < bed.1[0] {
                xs.push(x);
            }
        }
        for y in [ex.0[1], ex.1[1]] {
            if y > bed.0[1] && y < bed.1[1] {
                ys.push(y);
            }
        }
    }
    xs.sort_by(f64::total_cmp);
    ys.sort_by(f64::total_cmp);
    xs.dedup();
    ys.dedup();

    let min_size = 2.0 * SCENE_EPSILON;
    let y_spans = spans(&ys);
    let mut free_volumes = Vec::new();
    for (min_x, max_x) in spans(&xs) {
        for &(min_y, max_y) in &y_spans {
            if max_x - min_x <= min_size || max_y - min_y <= min_size {
                continue;
            }
            let blocked = excludes
                .iter()
                .any(|ex| min_x < ex.1[0] && max_x > ex.0[0] && min_y < ex.1[1] && max_y > ex.0[1]);
            if !blocked {
                free_volumes.push(([min_x, min_y, bed.0[2]], [max_x, max_y, bed.1[2]]));
            }
        }
    }
    free_volumes
}

/// The edge pairs `(edges[i], edges[j])`, `i < j`, in the loops' order.
fn spans(edges: &[f64]) -> Vec<(f64, f64)> {
    (0..edges.len())
        .flat_map(|i| edges[i + 1..].iter().map(move |&max| (edges[i], max)))
        .collect()
}

/// A volume's size over the selection's, zero for an empty selection extent.
fn ratio(volume_size: f64, box_size: f64) -> f64 {
    if box_size != 0.0 {
        volume_size / box_size
    } else {
        0.0
    }
}

/// `unscale<double>`.
fn unscale(value: f64) -> f64 {
    value * SCALING_FACTOR
}

/// `coord_t(scale_(value))`.
fn scaled(value: f64) -> i64 {
    (value / SCALING_FACTOR) as i64
}

/// `BuildVolume::bed_center()` (`BuildVolume.hpp:87`): the centre of the printable area's box.
fn bed_center(area: &[crate::Point2d]) -> [f64; 2] {
    let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
    for point in area {
        for (axis, value) in [point.x, point.y].into_iter().enumerate() {
            min[axis] = min[axis].min(value);
            max[axis] = max[axis].max(value);
        }
    }
    std::array::from_fn(|axis| 0.5 * (min[axis] + max[axis]))
}
