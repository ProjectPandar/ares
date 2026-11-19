use crate::{Project, SliceError, project::FaceChoice};

/// Model transforms in the order the CLI first encountered their keys.
#[derive(Clone, Debug)]
pub enum CliVolumeTransform {
    Scale(f64),
    Rotate(f64),
    GroundLargestFace(bool),
    GroundFaceNormal(String),
    GroundFacePoint(String),
}

/// Model actions from `OrcaSlicer.cpp:5051–5153,5271–5288,5619–6100`.
#[derive(Clone, Debug, Default)]
pub struct CliProjectTransforms {
    pub volume: Vec<CliVolumeTransform>,
    pub orient: bool,
    pub arrange: Option<i32>,
}

impl CliProjectTransforms {
    pub const NO_OBJECTS_AFTER_ARRANGE: &str = "No object is fully inside the print volume. Arrange ran without rotations. If an object only fits the bed when rotated, pass --allow-rotations. Otherwise the plate is empty or an object is larger than the printable area or height.";
    pub const NONE: Self = Self {
        volume: Vec::new(),
        orient: false,
        arrange: None,
    };

    pub fn has_project_actions(&self) -> bool {
        self.arrange.is_some() || !self.volume.is_empty()
    }

    pub(super) fn apply(
        &self,
        mut project: Project,
        plate: Option<u32>,
    ) -> Result<Project, SliceError> {
        if !self.volume.is_empty() {
            project.prepare_import_scope(plate);
        }
        for transform in &self.volume {
            match transform {
                CliVolumeTransform::Scale(ratio) => scale_volumes(&mut project, *ratio)?,
                CliVolumeTransform::Rotate(degrees) if !degrees.is_finite() => {
                    return Err(SliceError::InvalidInput("rotation must be finite".into()));
                }
                CliVolumeTransform::Rotate(degrees) => {
                    project.rotate_cli_volumes(degrees.to_radians());
                }
                CliVolumeTransform::GroundLargestFace(false) => (),
                CliVolumeTransform::GroundLargestFace(true) => {
                    project.ground_cli_face(FaceChoice::Largest)?;
                }
                CliVolumeTransform::GroundFaceNormal(value) => {
                    project.ground_cli_face(FaceChoice::parse(
                        "ground_face_normal",
                        value,
                        true,
                    )?)?;
                }
                CliVolumeTransform::GroundFacePoint(value) => {
                    project.ground_cli_face(FaceChoice::parse(
                        "ground_face_point",
                        value,
                        false,
                    )?)?;
                }
            }
        }
        if self.orient {
            crate::plater::apply_cli_orientation(&mut project, plate)?;
        }
        if self.forces_arrange() {
            project.prepare_import_scope(plate);
            return crate::plater::arrange_cli_project(project, plate);
        }
        Ok(project)
    }

    pub(super) fn forces_arrange(&self) -> bool {
        self.arrange == Some(1)
    }
}

// OrcaSlicer.cpp:5143 narrows the parsed double before both the check and the edit.
fn scale_volumes(project: &mut Project, ratio: f64) -> Result<(), SliceError> {
    let ratio = ratio as f32;
    if ratio <= 0.0 || !ratio.is_finite() {
        return Err(SliceError::InvalidCliScale(ratio));
    }
    project.scale_cli_volumes(f64::from(ratio));
    Ok(())
}

pub(super) fn with_arrange_diagnostic(error: SliceError, arranged: bool) -> SliceError {
    if arranged && matches!(&error, SliceError::NoSuitableObjects(_)) {
        SliceError::NoSuitableObjectsAfterCliArrange
    } else {
        error
    }
}
