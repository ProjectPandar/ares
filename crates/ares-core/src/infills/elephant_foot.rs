use crate::InfillOptions;

pub(super) fn internal_solid_spacing(options: &InfillOptions, layer_index: usize) -> Option<f64> {
    let density = options.elephant_foot_layers_density_percent();
    let layers = options.elephant_foot_compensation_layers();
    if density == 100.0 || layer_index == 0 || layer_index > layers {
        return Some(options.solid_line_width());
    }
    let density = density / 100.0;
    let layers = layers as f64;
    let layer_index = layer_index as f64;
    let density_percent = (1.0 - (1.0 - density) * (layers - (layer_index - 1.0)) / layers) * 100.0;
    Some(options.solid_line_width() / (density_percent / 100.0))
}
