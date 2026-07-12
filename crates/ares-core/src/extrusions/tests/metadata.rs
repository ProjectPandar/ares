use super::*;
use crate::{Layer, LayerToolpathMoves};

#[test]
fn preserves_empty_represented_layers_for_extrusions() {
    let layers = [Layer::new(7, 0.2, 1.4)];
    let moves = [LayerToolpathMoves::new(7, 1.4, Vec::new())];
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.2, (0.2, 0.0), 0.2);

    let output = generate_extrusion_moves(&layers, &moves, options).unwrap();

    assert_eq!(output.len(), 1);
    assert_eq!(output[0].layer_id(), 7);
    assert!(output[0].moves().is_empty());
}
