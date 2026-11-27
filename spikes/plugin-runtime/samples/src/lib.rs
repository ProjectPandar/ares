use plugin_spike_api::{Edit, Layer, OptionDeclaration, OptionKind, OptionScope, Plugin, Value};
use std::convert::Infallible;

pub enum Sample {
    Staggered { offset_adjustment: f64 },
    Subdivided { lower_scale: f64 },
}

impl Plugin for Sample {
    type Error = Infallible;

    fn options(&self) -> &[OptionDeclaration] {
        const OFFSET: &[OptionDeclaration] = &[OptionDeclaration {
            key: "odd_offset",
            label: "Odd perimeter Z offset",
            scope: OptionScope::Print,
            kind: OptionKind::Number { min: 0.0, max: 1.0 },
            default: Value::Number(0.5),
        }];
        const FRACTION: &[OptionDeclaration] = &[OptionDeclaration {
            key: "lower_fraction",
            label: "Lower pass height fraction",
            scope: OptionScope::Print,
            kind: OptionKind::Number {
                min: 0.05,
                max: 0.95,
            },
            default: Value::Number(0.5),
        }];
        match self {
            Self::Staggered { .. } => OFFSET,
            Self::Subdivided { .. } => FRACTION,
        }
    }

    fn perimeters(&mut self, layer: Layer<'_>) -> Result<Vec<Edit>, Infallible> {
        Ok(match *self {
            Self::Staggered { offset_adjustment } => {
                let Value::Number(offset) = layer.option("odd_offset").unwrap() else {
                    unreachable!()
                };
                staggered(layer, offset + offset_adjustment)
            }
            Self::Subdivided { lower_scale } => {
                let Value::Number(fraction) = layer.option("lower_fraction").unwrap() else {
                    unreachable!()
                };
                subdivided(layer, fraction * lower_scale)
            }
        })
    }

    fn gcode_post_process(&mut self, input: &[u8]) -> Result<Vec<u8>, Infallible> {
        let mut result = input.to_vec();
        result.extend_from_slice(b"; post processed\n");
        Ok(result)
    }
}

fn staggered(layer: Layer<'_>, offset: f64) -> Vec<Edit> {
    let top = layer.z_bottom + layer.height;
    let mut edits = Vec::with_capacity(layer.paths.len());
    for parity in 0..=1 {
        edits.extend(layer.paths.iter().enumerate().filter_map(|(source, path)| {
            (path.inset_index % 2 == parity).then_some(Edit {
                source,
                z: top + f64::from(parity) * offset,
                height: layer.height,
                flow_multiplier: 1.0,
            })
        }));
    }
    edits
}

fn subdivided(layer: Layer<'_>, lower_fraction: f64) -> Vec<Edit> {
    let lower = layer.height * lower_fraction;
    let top = layer.z_bottom + layer.height;
    let mut edits = Vec::with_capacity(layer.paths.len() * 2);
    edits.extend(layer.paths.iter().enumerate().filter_map(|(source, path)| {
        (path.inset_index == 0).then_some(Edit {
            source,
            z: layer.z_bottom + lower,
            height: lower,
            flow_multiplier: 1.0,
        })
    }));
    edits.extend(layer.paths.iter().enumerate().map(|(source, path)| Edit {
        source,
        z: top,
        height: if path.inset_index == 0 {
            layer.height - lower
        } else {
            layer.height
        },
        flow_multiplier: 1.0,
    }));
    edits
}
