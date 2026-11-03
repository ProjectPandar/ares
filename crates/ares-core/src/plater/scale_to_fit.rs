//! "Scale to build volume" (`Plater::priv::scale_selection_to_fit_print_volume`,
//! `Plater.cpp:11092-11112` at v2.5.0-beta `345efec64`; `Selection::scale_to_fit_print_volume`,
//! `Selection.cpp:1481-1580`). See `docs/architecture/web-plater-scale-to-fit.md`.

use super::PlaterSession;
use super::plate_list::{BoundingBox, InstanceKey};
use super::selection::{self, TransformationType};
use crate::SliceError;
use crate::project_slice::{BedShape, PrintVolume};

/// `BuildVolume::SceneEpsilon` (`BuildVolume.hpp:110`).
const SCENE_EPSILON: f64 = 1e-4;

/// An exclusion box's X and Y extent: `(min, max)`.
type ExcludeBox = ([f64; 2], [f64; 2]);

impl PlaterSession {
    /// Fits one full instance into the current plate's free build volume. On a Circle bed it is
    /// refused: `fit_circle` (`Selection.cpp:1534-1569`) is not ported.
    pub fn scale_to_fit(&mut self, object: usize, instance: usize) -> Result<(), SliceError> {
        self.check(object, instance)?;
        match PrintVolume::new(&self.project)?.shape() {
            BedShape::Rectangle { .. } => {}
            BedShape::Circle(_) => {
                return Err(SliceError::InvalidInput(
                    "scale to build volume is not ported for a circular bed".to_owned(),
                ));
            }
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

    /// Whether `scale_to_fit` is ported for the bed: every bed but a Circle one.
    pub fn scale_to_fit_ported(&self) -> Result<bool, SliceError> {
        Ok(!matches!(
            PrintVolume::new(&self.project)?.shape(),
            BedShape::Circle(_)
        ))
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
