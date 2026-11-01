//! The "Scale ratios" and "Size" fields of one instance in world coordinates
//! (`GizmoObjectManipulation.cpp:115-150, 408-490, 609-616` at v2.5.0-beta `345efec64`), through
//! `Selection::scale` (`Selection.cpp:1475-1478, 1627-1719`) with the instance sync (`:3045`) and
//! `GLCanvas3D::do_scale` (`GLCanvas3D.cpp:5889-5985`). See `docs/architecture/web-plater-scale.md`.

use super::PlaterSession;
use super::plate_list::InstanceKey;
use super::selection;
use super::session::{Model, check_axis, keys};
use crate::SliceError;

/// `MAX_NUM` (`GizmoObjectManipulation.cpp:45`).
const MAX_NUM: f64 = 9999.99;

/// `EPSILON` (`libslic3r.h:52`).
const EPSILON: f64 = 1e-4;

/// One field edit: the axis edited, its new value (percent for a scale, mm for a size) and the
/// panel's uniform toggle (`m_uniform_scale`, `GizmoObjectManipulation.hpp:99`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleEdit {
    pub axis: usize,
    pub value: f64,
    pub uniform: bool,
}

/// The panel's cached values for a single full instance in world coordinates
/// (`update_settings_value`, `GizmoObjectManipulation.cpp:132-134`).
struct Fields {
    scale: [f64; 3],
    size: [f64; 3],
    unscale_size: [f64; 3],
}

impl Fields {
    fn read(session: &PlaterSession, key: InstanceKey) -> Self {
        let size = selection::world_box_size(&session.project, key);
        let unscale_size = selection::unscaled_instance_box_size(&session.project, key, true);
        Self {
            scale: std::array::from_fn(|axis| size[axis] / unscale_size[axis] * 100.0),
            size,
            unscale_size,
        }
    }
}

/// `m_cache.*_rounded` (`GizmoObjectManipulation.cpp:251-252`): the value the field shows.
fn rounded(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

impl PlaterSession {
    /// The scale (percent) and size (mm) the fields show (`update_settings_value`, `:115-150`).
    pub fn scale_and_size(
        &self,
        object: usize,
        instance: usize,
    ) -> Result<([f64; 3], [f64; 3]), SliceError> {
        self.check(object, instance)?;
        let fields = Fields::read(self, (object, instance));
        Ok((fields.scale, fields.size))
    }

    /// A "Scale ratios" edit (`change_scale_value`, `:408-430`): a value at or below zero, or
    /// the one the field shows, changes nothing.
    pub fn set_scale(
        &mut self,
        object: usize,
        instance: usize,
        edit: ScaleEdit,
    ) -> Result<(), SliceError> {
        self.check(object, instance)?;
        let axis = check_axis(edit.axis)?;
        let fields = Fields::read(self, (object, instance));
        if edit.value <= 0.0 || (rounded(fields.scale[axis]) - edit.value).abs() < EPSILON {
            return Ok(());
        }
        let mut scale = fields.scale;
        scale[axis] = edit.value;
        let scale = scale.map(|value| value / 100.0);
        self.do_scale((object, instance), (edit, scale), fields.unscale_size);
        Ok(())
    }

    /// A "Size" edit (`change_size_value`, `:433-460`), relative to the full unscaled instance
    /// box in world coordinates.
    pub fn set_size(
        &mut self,
        object: usize,
        instance: usize,
        edit: ScaleEdit,
    ) -> Result<(), SliceError> {
        self.check(object, instance)?;
        let axis = check_axis(edit.axis)?;
        let fields = Fields::read(self, (object, instance));
        if edit.value <= 0.0 || (rounded(fields.size[axis]) - edit.value).abs() < EPSILON {
            return Ok(());
        }
        let mut size = fields.size;
        size[axis] = edit.value;
        let ref_size =
            selection::unscaled_instance_box_size(&self.project, (object, instance), false);
        let scale = std::array::from_fn(|axis| size[axis] / ref_size[axis]);
        self.do_scale((object, instance), (edit, scale), fields.unscale_size);
        Ok(())
    }

    /// `GizmoObjectManipulation::do_scale` (`:463-481`) with `limit_scaling_ratio` (`:484-490`),
    /// then `GLCanvas3D::do_scale` (`GLCanvas3D.cpp:5897-5962`): every instance's lowest point is
    /// kept from before the scale, the scaled object's instances are notified, and each instance
    /// drops onto the bed unless it stays sinking.
    fn do_scale(
        &mut self,
        key: InstanceKey,
        (edit, scale): (ScaleEdit, [f64; 3]),
        unscale_size: [f64; 3],
    ) {
        let mut scaling_factor = if edit.uniform {
            [scale[edit.axis]; 3]
        } else {
            scale
        };
        for (factor, unscale) in scaling_factor.iter_mut().zip(unscale_size) {
            if *factor * unscale > MAX_NUM {
                *factor = MAX_NUM / unscale;
            }
        }
        let instances: Vec<_> = keys(&self.project)
            .into_iter()
            .map(|key| {
                let min_z = selection::instance_bounding_box(&self.project, key).0[2];
                (key, min_z)
            })
            .collect();
        selection::scale(&mut self.project, key, scaling_factor);
        for instance in 0..self.project.objects()[key.0].instances().len() {
            self.plates
                .notify_instance_update(&Model(&mut self.project), (key.0, instance));
        }
        for (key, min_z) in instances {
            if selection::drop_after_transform(&mut self.project, key, min_z) {
                self.plates
                    .notify_instance_update(&Model(&mut self.project), key);
            }
        }
    }
}
