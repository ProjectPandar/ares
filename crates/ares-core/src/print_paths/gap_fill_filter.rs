use crate::{LayerPrintPaths, Point2, PrintPathRole};

pub fn filter_short_gap_fill_paths(
    layers: Vec<LayerPrintPaths>,
    filter_out_gap_fill_mm: f64,
) -> Vec<LayerPrintPaths> {
    if filter_out_gap_fill_mm <= 0.0 {
        return layers;
    }

    layers
        .into_iter()
        .map(|layer| {
            let paths = layer
                .paths
                .into_iter()
                .filter(|path| {
                    path.role != PrintPathRole::GapFill
                        || polyline_length(path.points()) >= filter_out_gap_fill_mm
                })
                .collect();
            LayerPrintPaths::new(layer.layer_id, layer.print_z, paths)
        })
        .collect()
}

fn polyline_length(points: &[Point2]) -> f64 {
    points
        .windows(2)
        .map(|segment| {
            let start = segment[0];
            let end = segment[1];
            ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
        })
        .sum()
}
