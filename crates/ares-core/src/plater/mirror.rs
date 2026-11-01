//! The object menu's "Mirror" (`GUI_Factories.cpp:1351-1364` at v2.5.0-beta `345efec64`):
//! `GLCanvas3D::mirror_selection` (`GLCanvas3D.cpp:3152-3162`), `Selection::mirror`
//! (`Selection.cpp:1708-1712`) and `GLCanvas3D::do_mirror` (`GLCanvas3D.cpp:6010-6092`). See
//! `docs/architecture/web-plater-mirror.md`.

use super::PlaterSession;
use crate::SliceError;

impl PlaterSession {
    /// Mirrors an instance along `axis` (0 X, 1 Y, 2 Z) about its centre (`Plater::mirror`).
    pub fn mirror(
        &mut self,
        _object: usize,
        _instance: usize,
        _axis: usize,
    ) -> Result<(), SliceError> {
        todo!("PlaterSession::mirror")
    }
}
