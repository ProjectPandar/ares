use super::{LayerPrintPaths, PrintPath, PrintPathRole, support_rectangle};

pub(crate) fn apply_support_critical_regions_only(
    layers: Vec<LayerPrintPaths>,
    enabled: bool,
) -> Vec<LayerPrintPaths> {
    if !enabled {
        return layers;
    }

    layers
        .into_iter()
        .map(|layer| {
            let paths = layer
                .paths()
                .iter()
                .filter(|path| keep_path(path))
                .cloned()
                .collect();
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect()
}

fn keep_path(path: &PrintPath) -> bool {
    if !support_path(path) || !path.is_closed() {
        return true;
    }
    support_rectangle::rectangle_bounds(path.points()).is_none()
}

fn support_path(path: &PrintPath) -> bool {
    matches!(
        path.role(),
        PrintPathRole::SupportMaterial | PrintPathRole::SupportMaterialInterface
    )
}
