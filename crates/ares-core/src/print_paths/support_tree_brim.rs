use super::{LayerPrintPaths, PrintPath, PrintPathRole, support_rectangle};

const AUTO_TREE_BRIM_WIDTH_MM: f64 = 2.0;

pub(crate) fn apply_tree_support_brim(
    layers: Vec<LayerPrintPaths>,
    is_tree_support: bool,
    raft_layers: u32,
    auto_brim: bool,
    manual_brim_width_mm: f64,
) -> Vec<LayerPrintPaths> {
    let Some(brim_width_mm) = tree_support_brim_width(
        is_tree_support,
        raft_layers,
        auto_brim,
        manual_brim_width_mm,
    ) else {
        return layers;
    };

    layers
        .into_iter()
        .map(|layer| {
            if layer.layer_id() != 0 {
                return layer;
            }

            let paths = layer
                .paths()
                .iter()
                .map(|path| expand_support_material_path(path, brim_width_mm))
                .collect();
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect()
}

fn tree_support_brim_width(
    is_tree_support: bool,
    raft_layers: u32,
    auto_brim: bool,
    manual_brim_width_mm: f64,
) -> Option<f64> {
    if !is_tree_support || raft_layers != 0 {
        return None;
    }
    if auto_brim {
        Some(AUTO_TREE_BRIM_WIDTH_MM)
    } else if manual_brim_width_mm > 0.0 {
        Some(manual_brim_width_mm)
    } else {
        None
    }
}

fn expand_support_material_path(path: &PrintPath, brim_width_mm: f64) -> PrintPath {
    if path.role() != PrintPathRole::SupportMaterial || !path.is_closed() {
        return path.clone();
    }

    let Some(bounds) = support_rectangle::rectangle_bounds(path.points()) else {
        return path.clone();
    };

    support_rectangle::rebuild_path(
        path,
        PrintPathRole::SupportMaterial,
        support_rectangle::rectangle_points(support_rectangle::RectangleBounds {
            min_x: bounds.min_x - brim_width_mm,
            min_y: bounds.min_y - brim_width_mm,
            max_x: bounds.max_x + brim_width_mm,
            max_y: bounds.max_y + brim_width_mm,
        }),
        true,
    )
}
