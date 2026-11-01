//! The "Scale ratios" and "Size" fields of one instance in world coordinates
//! (`GizmoObjectManipulation.cpp:115-150, 408-490, 609-616` at v2.5.0-beta `345efec64`), through
//! `Selection::scale` (`Selection.cpp:1475-1478, 1627-1719`) with the instance sync (`:3045`) and
//! `GLCanvas3D::do_scale` (`GLCanvas3D.cpp:5889-5985`). See `docs/architecture/web-plater-scale.md`.

use super::PlaterSession;
use crate::SliceError;

/// One field edit: the axis edited, its new value (percent for a scale, mm for a size) and the
/// panel's uniform toggle (`m_uniform_scale`, `GizmoObjectManipulation.hpp:99`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleEdit {
    pub axis: usize,
    pub value: f64,
    pub uniform: bool,
}

impl PlaterSession {
    /// The scale (percent) and size (mm) the fields show (`update_settings_value`, `:115-150`).
    pub fn scale_and_size(
        &self,
        _object: usize,
        _instance: usize,
    ) -> Result<([f64; 3], [f64; 3]), SliceError> {
        todo!("PlaterSession::scale_and_size")
    }

    /// A "Scale ratios" edit (`change_scale_value`, `:408-430`).
    pub fn set_scale(
        &mut self,
        _object: usize,
        _instance: usize,
        _edit: ScaleEdit,
    ) -> Result<(), SliceError> {
        todo!("PlaterSession::set_scale")
    }

    /// A "Size" edit (`change_size_value`, `:433-460`).
    pub fn set_size(
        &mut self,
        _object: usize,
        _instance: usize,
        _edit: ScaleEdit,
    ) -> Result<(), SliceError> {
        todo!("PlaterSession::set_size")
    }
}
