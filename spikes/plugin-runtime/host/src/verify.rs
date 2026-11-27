use plugin_spike_api::Edit;
use plugin_spike_wire::{LAYER_HEIGHT, PATHS, encode_edits};

pub fn first_layer(bytes: &[u8], subdivide: bool, reloaded: bool) {
    assert_eq!(bytes, expected(subdivide, reloaded));
}

pub fn expected(subdivide: bool, reloaded: bool) -> Vec<u8> {
    let indices: Vec<_> = if subdivide {
        (0..PATHS).step_by(4).chain(0..PATHS).collect()
    } else {
        (0..PATHS).step_by(2).chain((1..PATHS).step_by(2)).collect()
    };
    let mut edits = Vec::with_capacity(indices.len());
    for (rank, index) in indices.into_iter().enumerate() {
        let lower = LAYER_HEIGHT * if reloaded { 0.25 } else { 0.5 };
        let offset = if reloaded { 0.75 } else { 0.5 };
        let (z, height) = match (subdivide, rank, index) {
            (true, split, _) if split < PATHS / 4 => (lower, lower),
            (true, _, outer) if outer % 4 == 0 => (LAYER_HEIGHT, LAYER_HEIGHT - lower),
            (false, _, odd) if odd % 2 == 1 => (LAYER_HEIGHT + offset, LAYER_HEIGHT),
            _ => (LAYER_HEIGHT, LAYER_HEIGHT),
        };
        edits.push(Edit {
            source: index,
            z,
            height,
            flow_multiplier: 1.0,
        });
    }
    encode_edits(&edits)
}
