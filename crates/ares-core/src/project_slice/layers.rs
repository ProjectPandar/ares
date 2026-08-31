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

#[derive(Clone, Copy)]
pub(super) struct LayerPlanOptions {
    pub(super) precise_z_height: bool,
    pub(super) zaa_min_z: Option<f64>,
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

pub(super) fn adjust_layer_pairs_to_object_height(
    parameters: &SlicingParameters,
    pairs: &mut [LayerPair],
) -> Result<bool, SliceError> {
    let object_height = parameters.object_print_z_max - parameters.object_print_z_min;
    let Some(last) = pairs.last() else {
        return Ok(false);
    };
    if (last.hi - object_height).abs() < 1.0e-4 {
        return Ok(true);
    }
    if pairs.len() < 6 {
        return Ok(false);
    }
    let start = pairs.len() - 5;
    let mut heights = pairs[start..]
        .iter()
        .map(|pair| pair.hi - pair.lo)
        .collect::<Vec<_>>();
    let mut adjustable = [true; 5];
    let grow = last.hi < object_height;
    let mut gap = (last.hi - object_height).abs();
    while gap >= 1.0e-4 {
        let count = adjustable.iter().filter(|enabled| **enabled).count();
        if count == 0 {
            return Ok(false);
        }
        let delta = gap / count as f64;
        let mut remaining = 0.0;
        for (height, enabled) in heights.iter_mut().zip(&mut adjustable) {
            if !*enabled {
                continue;
            }
            let candidate = if grow {
                *height + delta
            } else {
                *height - delta
            };
            let boundary = if grow {
                parameters.max_layer_height
            } else {
                parameters.min_layer_height
            };
            let exceeds = if grow {
                candidate > boundary
            } else {
                candidate < boundary
            };
            if exceeds {
                remaining += (candidate - boundary).abs();
                *height = boundary;
                *enabled = false;
            } else {
                *height = candidate;
            }
        }
        require_finite(remaining)?;
        gap = remaining;
    }
    for index in 0..5 {
        if index > 0 {
            pairs[start + index].lo = pairs[start + index - 1].hi;
        }
        pairs[start + index].hi = pairs[start + index].lo + heights[index];
    }
    Ok(true)
}

#[cfg(test)]
pub(super) fn planned_layers(pairs: &[LayerPair]) -> Result<Vec<PlannedLayer>, SliceError> {
    planned_layers_with_zaa(pairs, None)
}

pub(super) fn planned_layers_with_zaa(
    pairs: &[LayerPair],
    zaa_min_z: Option<f64>,
) -> Result<Vec<PlannedLayer>, SliceError> {
    if pairs.is_empty() {
        return Err(invalid("object layer pair series is empty"));
    }
    pairs
        .iter()
        .enumerate()
        .map(|(id, pair)| {
            let height = pair.hi - pair.lo;
            let print_z = pair.hi;
            // OrcaSlicer 2.4.2 `PrintObjectSlice.cpp::compute_slice_z`.
            let slice_z = match (id, zaa_min_z) {
                (0, _) | (_, None) => 0.5 * (pair.lo + pair.hi),
                (_, Some(offset)) => pair.lo + offset,
            };
            require_finite(height)?;
            require_finite(print_z)?;
            require_finite(slice_z)?;
            if slice_z < pair.lo || slice_z > pair.hi {
                return Err(invalid("Bad min Z value"));
            }
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
    (source_object_index, transform_index): (usize, usize),
    parameters: &SlicingParameters,
    profile: &[f64],
    options: LayerPlanOptions,
    budget: &mut LayerBudget,
) -> Result<PlannedPrintObject, SliceError> {
    let mut pairs = generate_layer_pairs(parameters, profile, budget)?;
    if options.precise_z_height {
        adjust_layer_pairs_to_object_height(parameters, &mut pairs)?;
    }
    Ok(PlannedPrintObject {
        source_object_index,
        transform_index,
        layers: planned_layers_with_zaa(&pairs, options.zaa_min_z)?,
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

#[cfg(test)]
mod tests;
