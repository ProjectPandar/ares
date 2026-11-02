//! "Add instance" / "Remove instance" (`Plater::increase_instances` / `decrease_instances`,
//! `Plater.cpp:19289-19364` at v2.5.0-beta `345efec64`). See
//! `docs/architecture/web-plater-instance-copies.md`.

use super::PlaterSession;
use crate::SliceError;

impl PlaterSession {
    /// `increase_instances(num)` on the object at `object`.
    pub fn increase_instances(&mut self, object: usize, num: usize) -> Result<(), SliceError> {
        let _ = (object, num);
        todo!()
    }

    /// `decrease_instances(num)` on the object at `object`.
    pub fn decrease_instances(&mut self, object: usize, num: usize) -> Result<(), SliceError> {
        let _ = (object, num);
        todo!()
    }
}
