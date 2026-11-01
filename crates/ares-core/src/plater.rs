//! OrcaSlicer's plater model edits: instance move and rotate
//! (`Selection`, `GLCanvas3D::do_move` / `do_rotate`), the plate list
//! (`PartPlateList`) and per-plate slicing of the edited model. See
//! `docs/architecture/web-plater-move-rotate-arrange.md`.

mod arrange_job;
mod arrange_settings;
mod edits;
mod eigen;
mod min_sphere;
mod mirror;
mod plate_list;
mod scale;
mod selection;
mod session;
mod slice;
mod transformation;

pub use arrange_job::{ArrangeScope, ArrangeSettings};
pub use edits::PlaterEdits;
pub use scale::ScaleEdit;
pub use session::{PlaterPlate, PlaterSession};
pub use slice::{PlaterSlice, slice_plater_plate, slice_plater_with_profiles};

/// No edits: the loaded model as it is.
pub(crate) static NO_EDITS: PlaterEdits = PlaterEdits::NONE;
