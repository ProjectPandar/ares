//! OrcaSlicer's plater model edits: instance move and rotate
//! (`Selection`, `GLCanvas3D::do_move` / `do_rotate`), the plate list
//! (`PartPlateList`) and per-plate slicing of the edited model. See
//! `docs/architecture/web-plater-move-rotate-arrange.md`.

use crate::{
    ConfigOverrides, GenerationMetadata, ProcessedGCode, Project, SliceError, SlicerContext,
    Transform3d,
};

/// A loaded project the plater edits.
pub struct PlaterSession;

/// One `PartPlate` of the session.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaterPlate {
    /// The plate origin in the plater scene.
    pub origin: [f64; 2],
    /// `(object index, instance index)` pairs.
    pub instances: Vec<(usize, usize)>,
    /// The loaded plate (0-based) this plate is, `None` for an added plate.
    pub source: Option<usize>,
}

impl PlaterSession {
    pub fn load(_input: Vec<u8>) -> Result<Self, SliceError> {
        todo!("PlaterSession::load")
    }

    pub fn input(&self) -> &[u8] {
        todo!("PlaterSession::input")
    }

    pub fn project(&self) -> &Project {
        todo!("PlaterSession::project")
    }

    pub fn instance_transform(
        &self,
        _object: usize,
        _instance: usize,
    ) -> Result<Transform3d, SliceError> {
        todo!("PlaterSession::instance_transform")
    }

    pub fn plates(&self) -> Vec<PlaterPlate> {
        todo!("PartPlateList plates")
    }

    pub fn translate_instance(
        &mut self,
        _object: usize,
        _instance: usize,
        _displacement: [f64; 3],
    ) -> Result<(), SliceError> {
        todo!("Selection::translate + GLCanvas3D::do_move")
    }

    pub fn set_instance_position(
        &mut self,
        _object: usize,
        _instance: usize,
        _axis: usize,
        _value: f64,
    ) -> Result<(), SliceError> {
        todo!("GizmoObjectManipulation::change_position_value")
    }

    pub fn rotate_instance(
        &mut self,
        _object: usize,
        _instance: usize,
        _radians: [f64; 3],
    ) -> Result<(), SliceError> {
        todo!("Selection::rotate + GLCanvas3D::do_rotate")
    }

    pub fn set_absolute_rotation(
        &mut self,
        _object: usize,
        _instance: usize,
        _axis: usize,
        _degrees: f64,
    ) -> Result<(), SliceError> {
        todo!("GizmoObjectManipulation::change_absolute_rotation_value")
    }

    pub fn add_plate(&mut self) {
        todo!("PartPlateList::create_plate")
    }

    pub fn delete_plate(&mut self, _index: usize) -> bool {
        todo!("PartPlateList::delete_plate")
    }
}

/// How a session plate is sliced.
#[derive(Clone, Copy, Debug)]
pub struct PlaterSlice<'a> {
    /// The plate id (`plater_id`, 1-based); `None` slices the first plate.
    pub plate: Option<u32>,
    pub overrides: &'a ConfigOverrides,
    /// Run `Print::validate` as the adapter entries do.
    pub validate_print: bool,
}

/// Slices a session plate with the project's own presets.
pub async fn slice_plater_plate(
    _session: &PlaterSession,
    _slice: PlaterSlice<'_>,
    _metadata: GenerationMetadata,
    _context: SlicerContext,
) -> Result<ProcessedGCode, SliceError> {
    todo!("per-plate slicing of the edited model")
}
