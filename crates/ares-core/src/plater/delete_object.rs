//! The object menu's "Delete" of a whole object (`Plater::priv::delete_object_from_model`,
//! `Plater.cpp:10750-10791` at v2.5.0-beta `345efec64`). See
//! `docs/architecture/web-plater-delete-object.md`.

use super::PlaterSession;
use crate::SliceError;

impl PlaterSession {
    /// `delete_object_from_model(object)`.
    pub fn delete_object(&mut self, object: usize) -> Result<(), SliceError> {
        let _ = object;
        todo!()
    }
}
