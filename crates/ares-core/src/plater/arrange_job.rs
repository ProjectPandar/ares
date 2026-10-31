//! The GUI arrange job (`src/slic3r/GUI/Jobs/ArrangeJob.cpp` at v2.5.0-beta `345efec64`) on the
//! plater session: `prepare`, `process` and `finalize` over the plate list, running
//! `arrangement::arrange`.

use super::PlaterSession;
use crate::SliceError;

/// Which instances an arrange moves (`Job::JobPrepareState`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrangeScope {
    /// `PREPARE_STATE_DEFAULT`: every instance on an unlocked plate (`prepare_all`).
    All,
    /// `PREPARE_STATE_MENU`: the instances on the plate at this 0-based index, the others
    /// left where they are (`prepare_partplate`).
    CurrentPlate(usize),
}

/// `GLCanvas3D::ArrangeSettings` (`GLCanvas3D.hpp:603-614`) of the FFF settings set in use;
/// `is_seq_print` follows the global `print_sequence` (`GLCanvas3D.cpp:1563-1580`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArrangeSettings {
    /// mm; 0 spaces the items automatically.
    pub distance: f32,
    pub enable_rotation: bool,
    pub allow_multi_materials_on_same_plate: bool,
    pub avoid_extrusion_cali_region: bool,
    pub align_to_y_axis: bool,
}

impl PlaterSession {
    /// Arranges the `scope` instances (`ArrangeJob::prepare`, `process` and `finalize`,
    /// `ArrangeJob.cpp:632-965`).
    pub fn arrange(
        &mut self,
        _scope: ArrangeScope,
        _settings: ArrangeSettings,
    ) -> Result<(), SliceError> {
        todo!("ArrangeJob on the plater session")
    }
}
