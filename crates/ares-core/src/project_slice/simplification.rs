use std::mem;

use crate::{
    SliceError,
    geometry::{ClipperError, CoordinateScale, append_simplified_expolygon},
};

use super::closing::PostClosingPrintObject;

pub(super) fn simplification_tolerance(resolution: f64, scale: CoordinateScale) -> Option<f64> {
    (resolution > 0.001).then(|| f64::from((0.0025_f64 / scale.factor()) as f32))
}

pub(super) fn apply_project_simplification(
    objects: &mut [PostClosingPrintObject],
    resolution: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let Some(tolerance) = simplification_tolerance(resolution, scale) else {
        return Ok(());
    };

    for layer in objects
        .iter_mut()
        .flat_map(PostClosingPrintObject::volumes_mut)
        .flat_map(|volume| volume.layers_mut())
    {
        let input = mem::take(layer.expolygons_mut());
        let mut output = Vec::with_capacity(input.len());
        for expolygon in input {
            append_simplified_expolygon(expolygon, tolerance, &mut output)
                .map_err(map_clipper_error)?;
        }
        *layer.expolygons_mut() = output;
    }
    Ok(())
}

fn map_clipper_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "project simplification polygon coordinate is outside the supported Clipper range"
                .to_owned(),
        ),
        ClipperError::OpenPathMustBeSubject | ClipperError::OpenPathsRequirePolyTree => {
            unreachable!("closed clipping cannot produce open-path errors")
        }
    }
}
