use super::{
    LayerPrintPaths, PrintPath, PrintPathRole,
    support_rectangle::{
        EPSILON, RectangleBounds, rebuild_path, rectangle_bounds, rectangle_points,
    },
};

const SUPPORT_CLOSING_RADIUS_MM: f64 = 2.0;

pub(crate) fn apply_support_style_snug(
    layers: Vec<LayerPrintPaths>,
    enabled: bool,
) -> Vec<LayerPrintPaths> {
    if !enabled {
        return layers;
    }

    layers.into_iter().map(apply_layer).collect()
}

fn apply_layer(layer: LayerPrintPaths) -> LayerPrintPaths {
    let mut clusters = Vec::new();
    let mut kept = Vec::new();

    for (index, path) in layer.paths().iter().enumerate() {
        if let Some(bounds) = snug_rectangle_bounds(path) {
            merge_snug_rectangle(&mut clusters, index, path, bounds);
        } else {
            kept.push((index, path.clone()));
        }
    }

    if clusters.is_empty() {
        return layer;
    }

    kept.extend(clusters.into_iter().map(|cluster| {
        let path = rebuild_path(
            &cluster.source,
            PrintPathRole::SupportMaterial,
            rectangle_points(cluster.bounds),
            true,
        );
        (cluster.first_index, path)
    }));
    kept.sort_by_key(|(index, _)| *index);
    let paths = kept.into_iter().map(|(_, path)| path).collect();

    LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
}

fn snug_rectangle_bounds(path: &PrintPath) -> Option<RectangleBounds> {
    if path.role() != PrintPathRole::SupportMaterial || !path.is_closed() {
        return None;
    }

    rectangle_bounds(path.points())
}

fn merge_snug_rectangle(
    clusters: &mut Vec<SnugCluster>,
    index: usize,
    path: &PrintPath,
    bounds: RectangleBounds,
) {
    let mut merged = SnugCluster {
        first_index: index,
        bounds,
        member_bounds: vec![bounds],
        source: path.clone(),
    };
    let mut cluster_index = 0;
    while cluster_index < clusters.len() {
        if merged.overlaps_member(&clusters[cluster_index]) {
            merged = merged.merge(clusters.remove(cluster_index));
            cluster_index = 0;
        } else {
            cluster_index += 1;
        }
    }
    clusters.push(merged);
}

fn inflated_bounds_overlap(left: RectangleBounds, right: RectangleBounds) -> bool {
    axis_overlap(
        left.min_x - SUPPORT_CLOSING_RADIUS_MM,
        left.max_x + SUPPORT_CLOSING_RADIUS_MM,
        right.min_x - SUPPORT_CLOSING_RADIUS_MM,
        right.max_x + SUPPORT_CLOSING_RADIUS_MM,
    ) && axis_overlap(
        left.min_y - SUPPORT_CLOSING_RADIUS_MM,
        left.max_y + SUPPORT_CLOSING_RADIUS_MM,
        right.min_y - SUPPORT_CLOSING_RADIUS_MM,
        right.max_y + SUPPORT_CLOSING_RADIUS_MM,
    )
}

fn axis_overlap(left_min: f64, left_max: f64, right_min: f64, right_max: f64) -> bool {
    left_max.min(right_max) - left_min.max(right_min) > EPSILON
}

#[derive(Clone)]
struct SnugCluster {
    first_index: usize,
    bounds: RectangleBounds,
    member_bounds: Vec<RectangleBounds>,
    source: PrintPath,
}

impl SnugCluster {
    fn overlaps_member(&self, other: &Self) -> bool {
        self.member_bounds.iter().any(|left| {
            other
                .member_bounds
                .iter()
                .any(|right| inflated_bounds_overlap(*left, *right))
        })
    }

    fn merge(mut self, other: Self) -> Self {
        if other.first_index < self.first_index {
            self.first_index = other.first_index;
            self.source = other.source;
        }
        self.bounds = union_bounds(self.bounds, other.bounds);
        self.member_bounds.extend(other.member_bounds);
        self
    }
}

fn union_bounds(left: RectangleBounds, right: RectangleBounds) -> RectangleBounds {
    RectangleBounds {
        min_x: left.min_x.min(right.min_x),
        min_y: left.min_y.min(right.min_y),
        max_x: left.max_x.max(right.max_x),
        max_y: left.max_y.max(right.max_y),
    }
}
