use super::{LayerPrintPaths, PrintPath, PrintPathRole, support_rectangle};

pub(crate) fn apply_support_remove_small_overhang(
    layers: Vec<LayerPrintPaths>,
    enabled: bool,
    line_width_mm: f64,
) -> Vec<LayerPrintPaths> {
    if !enabled {
        return layers;
    }

    let min_size = 4.0 * line_width_mm;
    layers
        .into_iter()
        .map(|layer| {
            let paths = layer
                .paths()
                .iter()
                .filter(|path| keep_path(path, min_size))
                .cloned()
                .collect();
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect()
}

fn keep_path(path: &PrintPath, min_size: f64) -> bool {
    if !support_path(path) || !path.is_closed() {
        return true;
    }
    let Some(bounds) = support_rectangle::rectangle_bounds(path.points()) else {
        return true;
    };

    let width = bounds.max_x - bounds.min_x;
    let height = bounds.max_y - bounds.min_y;
    width + support_rectangle::EPSILON >= min_size
        && height + support_rectangle::EPSILON >= min_size
}

fn support_path(path: &PrintPath) -> bool {
    matches!(
        path.role(),
        PrintPathRole::SupportMaterial | PrintPathRole::SupportMaterialInterface
    )
}
