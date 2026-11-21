//! OrcaSlicer's plater model edits: instance move and rotate
//! (`Selection`, `GLCanvas3D::do_move` / `do_rotate`), the plate list
//! (`PartPlateList`) and per-plate slicing of the edited model. See
//! `docs/architecture/web-plater-move-rotate-arrange.md`.

mod auto_orient;
pub(crate) use auto_orient::apply_cli_orientation;
pub use auto_orient::{
    CoolingOrientation, OrientMesh, OrientParams, Orientation, auto_orient, auto_orient_mesh,
};
pub use auto_orient::{auto_orient_instance, auto_orient_object};
mod arrange_job;
mod arrange_settings;
mod current_plate;
mod delete_object;
mod edits;
mod eigen;
mod initial_plate;
mod instances;
mod min_sphere;
mod mirror;
mod plate_list;
mod plate_lock;
mod scale;
mod scale_to_fit;
mod selection;
mod session;
mod slice;
mod slice_tower_fit;
mod snapshot;
mod stl_files;
pub use stl_files::{StlFile, StlImportOptions};
pub(crate) mod transformation;

pub(crate) use arrange_job::arrange_cli_stl;
pub use arrange_job::{ArrangeScope, ArrangeSettings};
pub use edits::PlaterEdits;
pub(crate) use initial_plate::initialize_slice_plates;
pub use scale::ScaleEdit;
pub use session::{PlaterPlate, PlaterSession};
pub use slice::{PlaterSlice, slice_plater_plate, slice_plater_with_profiles};
pub(crate) use slice_tower_fit::fit_slice_wipe_tower;
pub use snapshot::PlaterSnapshot;

/// No edits: the loaded model as it is.
pub(crate) static NO_EDITS: PlaterEdits = PlaterEdits::NONE;
