use super::{LayerPrintPaths, PrintPath, PrintPathRole, support_rectangle};

pub(crate) fn apply_support_on_build_plate_only(
    layers: Vec<LayerPrintPaths>,
    enabled: bool,
    raft_layers: u32,
) -> Vec<LayerPrintPaths> {
    if !enabled {
        return layers;
    }

    let mut previous_retained = Vec::new();
    layers
        .into_iter()
        .map(|layer| {
            let anchor_layer = layer.layer_id() == 0 || layer.layer_id() < raft_layers as usize;
            let paths = layer
                .paths()
                .iter()
                .filter(|path| keep_path(path, anchor_layer, &previous_retained))
                .cloned()
                .collect::<Vec<_>>();
            previous_retained = retained_support_bounds(&paths);
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect()
}

fn keep_path(
    path: &PrintPath,
    anchor_layer: bool,
    previous_retained: &[support_rectangle::RectangleBounds],
) -> bool {
    if !support_path(path) || !path.is_closed() {
        return true;
    }
    let Some(bounds) = support_rectangle::rectangle_bounds(path.points()) else {
        return true;
    };

    anchor_layer
        || previous_retained
            .iter()
            .any(|lower| overlaps(bounds, *lower))
}

fn retained_support_bounds(paths: &[PrintPath]) -> Vec<support_rectangle::RectangleBounds> {
    paths
        .iter()
        .filter(|path| support_path(path) && path.is_closed())
        .filter_map(|path| support_rectangle::rectangle_bounds(path.points()))
        .collect()
}

fn support_path(path: &PrintPath) -> bool {
    matches!(
        path.role(),
        PrintPathRole::SupportMaterial | PrintPathRole::SupportMaterialInterface
    )
}

fn overlaps(
    current: support_rectangle::RectangleBounds,
    lower: support_rectangle::RectangleBounds,
) -> bool {
    current.max_x - lower.min_x > support_rectangle::EPSILON
        && lower.max_x - current.min_x > support_rectangle::EPSILON
        && current.max_y - lower.min_y > support_rectangle::EPSILON
        && lower.max_y - current.min_y > support_rectangle::EPSILON
}
