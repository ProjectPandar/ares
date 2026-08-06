use crate::{
    RegionOptions,
    project_slice::{
        layers::PlannedLayer, prepare_infill::horizontal_shell_propagation::types::SourceKind,
    },
};

const EPSILON: f64 = 1e-4;

pub(super) enum NeighborIndices {
    Lower(std::iter::Rev<std::ops::Range<usize>>),
    Upper(std::ops::Range<usize>),
}

impl Iterator for NeighborIndices {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Lower(indices) => indices.next(),
            Self::Upper(indices) => indices.next(),
        }
    }
}

pub(super) fn indices(kind: SourceKind, source: usize, len: usize) -> NeighborIndices {
    match kind {
        SourceKind::Top => NeighborIndices::Lower((0..source).rev()),
        SourceKind::Bottom | SourceKind::BottomBridge => NeighborIndices::Upper(source + 1..len),
    }
}

pub(super) const fn source_enabled(count: i32) -> bool {
    count != 0
}

pub(super) fn shell_count(kind: SourceKind, options: &RegionOptions) -> i32 {
    match kind {
        SourceKind::Top => options.top_shell_layers.0,
        SourceKind::Bottom | SourceKind::BottomBridge => options.bottom_shell_layers.0,
    }
}

pub(super) fn includes(
    kind: SourceKind,
    indices: [usize; 2],
    layers: &[PlannedLayer],
    count: i32,
    options: &RegionOptions,
) -> bool {
    let [source, neighbor] = indices;
    match kind {
        SourceKind::Top => {
            let distance = (source - neighbor) as i32;
            distance < count
                || layers[source].print_z - layers[neighbor].print_z
                    < options.top_shell_thickness.0 - EPSILON
        }
        SourceKind::Bottom | SourceKind::BottomBridge => {
            let distance = (neighbor - source) as i32;
            let source_bottom = layers[source].print_z - layers[source].height;
            let neighbor_bottom = layers[neighbor].print_z - layers[neighbor].height;
            distance < count
                || neighbor_bottom - source_bottom < options.bottom_shell_thickness.0 - EPSILON
        }
    }
}
