use crate::{InfillOptions, Layer};

use super::LayerInfills;

pub(super) fn apply(print_layers: &[Layer], infills: &mut [LayerInfills], options: &InfillOptions) {
    if !options.infill_combination() || options.sparse_density_percent() == 0.0 {
        return;
    }

    for (target, count) in combination_targets(
        print_layers,
        options.infill_combination_max_layer_height_mm(),
    )
    .into_iter()
    .enumerate()
    {
        if count <= 1 {
            continue;
        }
        let start = target + 1 - count;
        let combined_height = print_layers[start..=target]
            .iter()
            .map(Layer::height)
            .sum::<f64>();
        for layer in &mut infills[start..target] {
            layer.paths.clear();
        }
        for path in &mut infills[target].paths {
            path.effective_layer_height_mm = combined_height;
        }
    }
}

fn combination_targets(print_layers: &[Layer], max_height: f64) -> Vec<usize> {
    let mut combine = vec![0; print_layers.len()];
    let mut current_height = 0.0;
    let mut num_layers = 0;
    for (index, layer) in print_layers.iter().enumerate() {
        if layer.id() == 0 {
            continue;
        }
        if current_height + layer.height() >= max_height + f64::EPSILON {
            combine[index - 1] = num_layers;
            current_height = 0.0;
            num_layers = 0;
        }
        current_height += layer.height();
        num_layers += 1;
    }
    if !print_layers.is_empty() {
        let last = print_layers.len() - 1;
        combine[last] = num_layers;
    }
    combine
}
