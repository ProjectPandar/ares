//! The object menu's "Mirror" (`GUI_Factories.cpp:1351-1364` at v2.5.0-beta `345efec64`):
//! `GLCanvas3D::mirror_selection` (`GLCanvas3D.cpp:3152-3162`), `Selection::mirror`
//! (`Selection.cpp:1708-1712`) and `GLCanvas3D::do_mirror` (`GLCanvas3D.cpp:6010-6092`). See
//! `docs/architecture/web-plater-mirror.md`.

use super::PlaterSession;
use super::selection::TransformationType;
use super::session::{Model, check_axis, keys};
use crate::SliceError;

impl PlaterSession {
    /// Mirrors an instance along `axis` (0 X, 1 Y, 2 Z) about its centre (`Plater::mirror`): a
    /// relative world scale by -1 on the axis, then `do_mirror`, which also notifies the plate list
    /// of every instance (`:6077-6079`).
    pub fn mirror(
        &mut self,
        object: usize,
        instance: usize,
        axis: usize,
    ) -> Result<(), SliceError> {
        self.check(object, instance)?;
        let axis = check_axis(axis)?;
        let mirror = std::array::from_fn(|index| if index == axis { -1.0 } else { 1.0 });
        self.scale_and_drop((object, instance), (mirror, TransformationType::Relative));
        for key in keys(&self.project) {
            self.plates
                .notify_instance_update(&Model(&mut self.project), key);
        }
        Ok(())
    }
}
