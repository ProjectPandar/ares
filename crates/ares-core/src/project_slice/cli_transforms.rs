use crate::{Project, SliceError};

/// Volume operations in the order the CLI first encountered their keys.
#[derive(Clone, Copy, Debug)]
pub enum CliVolumeTransform {
    Scale(f64),
    Rotate(f64),
}

/// Explicit CLI arrangement inputs (`OrcaSlicer.cpp:1597–1626,1670`).
#[derive(Clone, Copy, Debug, Default)]
pub struct CliArrangeModifiers {
    pub allow_rotations: Option<bool>,
    pub align_to_y_axis: Option<bool>,
    pub allow_multicolor_oneplate: Option<bool>,
    pub avoid_extrusion_cali_region: Option<bool>,
}

/// Model actions from `OrcaSlicer.cpp:5051–5153,5271–5288,5619–6100`.
#[derive(Clone, Debug, Default)]
pub struct CliProjectTransforms {
    pub volume: Vec<CliVolumeTransform>,
    pub orient: bool,
    pub arrange: Option<i32>,
    pub arrange_modifiers: CliArrangeModifiers,
}

impl CliProjectTransforms {
    pub const NO_OBJECTS_AFTER_ARRANGE: &str = "No object is fully inside the print volume. Arrange ran without rotations. If an object only fits the bed when rotated, pass --allow-rotations. Otherwise the plate is empty or an object is larger than the printable area or height.";
    pub const NONE: Self = Self {
        volume: Vec::new(),
        orient: false,
        arrange: None,
        arrange_modifiers: CliArrangeModifiers {
            allow_rotations: None,
            align_to_y_axis: None,
            allow_multicolor_oneplate: None,
            avoid_extrusion_cali_region: None,
        },
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
        for &transform in &self.volume {
            match transform {
                CliVolumeTransform::Scale(ratio) => scale_volumes(&mut project, ratio)?,
                CliVolumeTransform::Rotate(degrees) if !degrees.is_finite() => {
                    return Err(SliceError::InvalidInput("rotation must be finite".into()));
                }
                CliVolumeTransform::Rotate(degrees) => {
                    project.rotate_cli_volumes(degrees.to_radians());
                }
            }
        }
        if self.orient {
            crate::plater::apply_cli_orientation(&mut project, plate)?;
        }
        if self.arrange == Some(1) {
            project.prepare_import_scope(plate);
            return crate::plater::arrange_cli_project(project, plate, self.arrange_modifiers);
        }
        Ok(project)
    }

    pub(super) fn arranges_without_rotations(&self) -> bool {
        self.arrange == Some(1) && !self.arrange_modifiers.allow_rotations.unwrap_or(false)
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

pub(super) fn with_arrange_diagnostic(
    error: SliceError,
    arranged_without_rotations: bool,
) -> SliceError {
    if arranged_without_rotations && matches!(&error, SliceError::NoSuitableObjects(_)) {
        SliceError::NoSuitableObjectsAfterCliArrange
    } else {
        error
    }
}
