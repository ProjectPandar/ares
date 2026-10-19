use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub(super) fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(super) fn moves(bytes: &[u8], wanted_role: &str) -> Vec<(usize, String)> {
    let mut layer = 0;
    let mut role = "";
    let mut result = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if line == ";LAYER_CHANGE" {
            layer += 1;
            role = "";
        }
        if let Some(next) = line.strip_prefix(";TYPE:") {
            role = next;
        }
        if role == wanted_role && (line.starts_with("G0 ") || line.starts_with("G1 ")) {
            result.push((layer, line.to_owned()));
        }
    }
    result
}

pub(super) fn positive_layers(rows: &[(usize, String)]) -> BTreeSet<usize> {
    rows.iter()
        .filter(|(_, line)| {
            let xy = line
                .split_ascii_whitespace()
                .any(|token| token.starts_with('X') || token.starts_with('Y'));
            let positive_e = line.split_ascii_whitespace().any(|token| {
                token
                    .strip_prefix('E')
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            });
            xy && positive_e
        })
        .map(|(layer, _)| *layer)
        .collect()
}
