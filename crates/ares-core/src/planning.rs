use crate::{Model, SliceError, SliceOptions};

const EPSILON: f64 = 1e-6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layer {
    id: usize,
    height: f64,
    print_z: f64,
}

#[derive(Clone, Copy, Debug)]
struct LayerHeightBounds {
    min: f64,
    max: f64,
}

impl Layer {
    pub const fn new(id: usize, height: f64, print_z: f64) -> Self {
        Self {
            id,
            height,
            print_z,
        }
    }

    pub const fn id(&self) -> usize {
        self.id
    }

    pub const fn height(&self) -> f64 {
        self.height
    }

    pub const fn print_z(&self) -> f64 {
        self.print_z
    }
}

pub fn plan_layers(model: &Model, options: &SliceOptions) -> Result<Vec<Layer>, SliceError> {
    let bounds = model
        .z_bounds()
        .ok_or_else(|| SliceError::InvalidInput("model has no triangles".to_owned()))?;
    let z_min = round_6(f64::from(bounds.min));
    let z_max = round_6(f64::from(bounds.max));
    if !z_min.is_finite() || !z_max.is_finite() || z_max <= z_min {
        return Err(SliceError::InvalidInput(
            "model must have positive Z height".to_owned(),
        ));
    }

    let layer_height = options.layer_height()?;
    let initial_layer_height = options.initial_layer_print_height()?;
    let planned_z_max =
        round_6(z_min + (z_max - z_min) * options.filament_shrinkage_compensation_z()?);
    let mut layers = Vec::new();
    let mut previous_z = z_min;
    let mut print_z = round_6(z_min + initial_layer_height).min(planned_z_max);

    loop {
        let height = print_z - previous_z;
        if height <= EPSILON {
            break;
        }
        layers.push(Layer::new(layers.len(), round_6(height), round_6(print_z)));
        previous_z = print_z;

        if round_6(previous_z + 0.5 * layer_height) >= planned_z_max - EPSILON {
            break;
        }

        print_z = round_6(previous_z + layer_height);
    }

    if layers.is_empty() {
        return Err(SliceError::InvalidInput(
            "model produced no layers".to_owned(),
        ));
    }

    if options.precise_z_height()? {
        let nozzle_min_layer_height = nozzle_min_layer_height(options)?;
        let min_layer_height = nozzle_min_layer_height.min(layer_height);
        let max_layer_height =
            effective_max_layer_height(options, nozzle_min_layer_height, layer_height)?;
        align_last_layers_to_object_height(
            &mut layers,
            z_min,
            planned_z_max,
            min_layer_height,
            max_layer_height,
        );
    }

    Ok(layers)
}

fn nozzle_min_layer_height(options: &SliceOptions) -> Result<f64, SliceError> {
    let configured = options.min_layer_heights()?[0];
    Ok(if configured == 0.0 {
        0.07
    } else {
        configured.max(0.01)
    })
}

fn effective_max_layer_height(
    options: &SliceOptions,
    nozzle_min_layer_height: f64,
    layer_height: f64,
) -> Result<f64, SliceError> {
    let configured = options.max_layer_heights()?[0];
    let nozzle_diameter = options.nozzle_diameters()?[0];
    let max_layer_height = if configured == 0.0 {
        0.75 * nozzle_diameter
    } else {
        configured
    };
    Ok(max_layer_height
        .max(nozzle_min_layer_height)
        .max(layer_height))
}

fn align_last_layers_to_object_height(
    layers: &mut [Layer],
    z_min: f64,
    z_max: f64,
    min_layer_height: f64,
    max_layer_height: f64,
) -> bool {
    let Some(last) = layers.last() else {
        return false;
    };
    if (last.print_z() - z_max).abs() <= EPSILON {
        return false;
    }
    if layers.len() < 6 {
        return false;
    }

    let start = layers.len() - 5;
    let mut heights: Vec<f64> = layers[start..].iter().map(Layer::height).collect();
    let mut can_adjust = [true; 5];
    let mut gap = (layers.last().unwrap().print_z() - z_max).abs();
    let needs_taller_layers = layers.last().unwrap().print_z() < z_max;

    while gap > EPSILON {
        let valid_count = can_adjust.iter().filter(|can| **can).count();
        if valid_count == 0 {
            return false;
        }

        let delta = gap / valid_count as f64;
        let mut remaining_gap = 0.0;
        for (index, height) in heights.iter_mut().enumerate() {
            remaining_gap += adjust_layer_height(
                height,
                &mut can_adjust[index],
                delta,
                needs_taller_layers,
                LayerHeightBounds {
                    min: min_layer_height,
                    max: max_layer_height,
                },
            );
        }
        gap = remaining_gap;
    }

    let mut previous_z = if start == 0 {
        z_min
    } else {
        layers[start - 1].print_z()
    };
    for (offset, height) in heights.into_iter().enumerate() {
        let index = start + offset;
        let print_z = round_6(previous_z + height);
        layers[index] = Layer::new(index, round_6(height), print_z);
        previous_z = print_z;
    }
    true
}

fn adjust_layer_height(
    height: &mut f64,
    can_adjust: &mut bool,
    delta: f64,
    needs_taller_layer: bool,
    bounds: LayerHeightBounds,
) -> f64 {
    if !*can_adjust {
        return 0.0;
    }

    if needs_taller_layer {
        return grow_layer_height(height, can_adjust, delta, bounds.max);
    }

    shrink_layer_height(height, can_adjust, delta, bounds.min)
}

fn grow_layer_height(
    height: &mut f64,
    can_adjust: &mut bool,
    delta: f64,
    max_layer_height: f64,
) -> f64 {
    let adjusted = *height + delta;
    if (*height - max_layer_height).abs() <= EPSILON {
        *can_adjust = false;
        return delta;
    }
    if adjusted <= max_layer_height {
        *height = adjusted;
        return 0.0;
    }

    *height = max_layer_height;
    *can_adjust = false;
    adjusted - max_layer_height
}

fn shrink_layer_height(
    height: &mut f64,
    can_adjust: &mut bool,
    delta: f64,
    min_layer_height: f64,
) -> f64 {
    let adjusted = *height - delta;
    if (*height - min_layer_height).abs() <= EPSILON {
        *can_adjust = false;
        return delta;
    }
    if adjusted >= min_layer_height {
        *height = adjusted;
        return 0.0;
    }

    *height = min_layer_height;
    *can_adjust = false;
    min_layer_height - adjusted
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests;
