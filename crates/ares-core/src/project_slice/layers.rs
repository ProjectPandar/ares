use crate::SliceError;

use super::parameters::SlicingParameters;

const MAX_PLANNED_LAYERS_PER_PROJECT: usize = 100_000;
const LIMIT_ERROR: &str = "project layer count exceeds supported limit of 100000";
const PROGRESS_ERROR: &str = "layer_height does not advance print_z";
const NONFINITE_ERROR: &str = "nonfinite layer generation value";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LayerPair {
    pub(super) lo: f64,
    pub(super) hi: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PlannedLayer {
    pub(super) id: usize,
    pub(super) height: f64,
    pub(super) print_z: f64,
    pub(super) slice_z: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PlannedPrintObject {
    pub(super) source_object_index: usize,
    pub(super) transform_index: usize,
    pub(super) layers: Vec<PlannedLayer>,
}

#[derive(Default)]
pub(super) struct LayerBudget {
    pub(super) used: usize,
}

impl LayerBudget {
    fn claim(&mut self) -> Result<(), SliceError> {
        if self.used >= MAX_PLANNED_LAYERS_PER_PROJECT {
            return Err(invalid(LIMIT_ERROR));
        }
        self.used += 1;
        Ok(())
    }
}

pub(super) fn generate_layer_pairs(
    parameters: &SlicingParameters,
    profile: &[f64],
    budget: &mut LayerBudget,
) -> Result<Vec<LayerPair>, SliceError> {
    budget.claim()?;
    let mut print_z = parameters.first_object_layer_height;
    let mut pairs = vec![LayerPair {
        lo: 0.0,
        hi: print_z,
    }];
    let object_height = parameters.object_print_z_max - parameters.object_print_z_min;
    let shrinkage_z = parameters.object_shrinkage_compensation_z;
    let mut profile_index = 0;
    let mut probe = print_z + 0.5 * parameters.min_layer_height;
    require_finite(probe)?;

    while probe < object_height {
        let height = profile_height(parameters, profile, &mut profile_index, probe, shrinkage_z)?;
        require_finite(height)?;
        let candidate_midpoint = print_z + 0.5 * height;
        require_finite(candidate_midpoint)?;
        if candidate_midpoint >= object_height {
            break;
        }

        let next_print_z = print_z + height;
        require_finite(next_print_z)?;
        if next_print_z <= print_z {
            return Err(invalid(PROGRESS_ERROR));
        }
        budget.claim()?;
        pairs.push(LayerPair {
            lo: print_z,
            hi: next_print_z,
        });
        print_z = next_print_z;
        probe = print_z + 0.5 * parameters.min_layer_height;
        require_finite(probe)?;
    }
    Ok(pairs)
}

fn profile_height(
    parameters: &SlicingParameters,
    profile: &[f64],
    profile_index: &mut usize,
    probe: f64,
    shrinkage_z: f64,
) -> Result<f64, SliceError> {
    if *profile_index >= profile.len() {
        return Ok(parameters.min_layer_height);
    }
    let mut next = *profile_index + 2;
    while next < profile.len() {
        let next_z = profile[next] * shrinkage_z;
        require_finite(next_z)?;
        if probe < next_z {
            break;
        }
        *profile_index = next;
        next += 2;
    }

    let z1 = profile[*profile_index] * shrinkage_z;
    let h1 = profile[*profile_index + 1];
    require_finite(z1)?;
    require_finite(h1)?;
    if next >= profile.len() {
        return Ok(h1);
    }
    let z2 = profile[next] * shrinkage_z;
    let h2 = profile[next + 1];
    require_finite(z2)?;
    require_finite(h2)?;
    let position = (probe - z1) / (z2 - z1);
    require_finite(position)?;
    Ok((1.0 - position) * h1 + position * h2)
}

pub(super) fn planned_layers(pairs: &[LayerPair]) -> Result<Vec<PlannedLayer>, SliceError> {
    if pairs.is_empty() {
        return Err(invalid("object layer pair series is empty"));
    }
    pairs
        .iter()
        .enumerate()
        .map(|(id, pair)| {
            let height = f64::from(pair.hi as f32 - pair.lo as f32);
            let print_z = pair.hi;
            let slice_z = 0.5 * (pair.lo + pair.hi);
            require_finite(height)?;
            require_finite(print_z)?;
            require_finite(slice_z)?;
            Ok(PlannedLayer {
                id,
                height,
                print_z,
                slice_z,
            })
        })
        .collect()
}

pub(super) fn plan_print_object(
    source_object_index: usize,
    transform_index: usize,
    parameters: &SlicingParameters,
    profile: &[f64],
    budget: &mut LayerBudget,
) -> Result<PlannedPrintObject, SliceError> {
    let pairs = generate_layer_pairs(parameters, profile, budget)?;
    Ok(PlannedPrintObject {
        source_object_index,
        transform_index,
        layers: planned_layers(&pairs)?,
    })
}

fn require_finite(value: f64) -> Result<(), SliceError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(invalid(NONFINITE_ERROR))
    }
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}
