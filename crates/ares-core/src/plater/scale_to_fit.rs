//! "Scale to build volume" (`Plater::priv::scale_selection_to_fit_print_volume`,
//! `Plater.cpp:11092-11112` at v2.5.0-beta `345efec64`; `Selection::scale_to_fit_print_volume`,
//! `Selection.cpp:1481-1580`). See `docs/architecture/web-plater-scale-to-fit.md`.

use super::PlaterSession;
use crate::SliceError;

impl PlaterSession {
    /// Fits one full instance into the current plate's free build volume.
    pub fn scale_to_fit(&mut self, object: usize, instance: usize) -> Result<(), SliceError> {
        todo!("scale_to_fit({object}, {instance})")
    }
}
