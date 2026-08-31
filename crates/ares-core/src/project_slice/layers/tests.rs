use super::{LayerPair, planned_layers_with_zaa};
use crate::SliceError;

fn pairs() -> [LayerPair; 3] {
    [
        LayerPair { lo: 0.0, hi: 0.2 },
        LayerPair { lo: 0.2, hi: 0.4 },
        LayerPair { lo: 0.4, hi: 0.6 },
    ]
}

#[test]
fn zaa_uses_midpoint_on_first_layer_and_minimum_offset_afterward() {
    let layers = planned_layers_with_zaa(&pairs(), Some(0.05)).unwrap();

    assert_eq!(layers[0].slice_z, 0.1);
    assert_eq!(layers[1].slice_z, 0.25);
    assert_eq!(layers[2].slice_z, 0.45);
}

#[test]
fn zaa_rejects_minimum_offset_outside_layer_interval() {
    assert_eq!(
        planned_layers_with_zaa(&pairs(), Some(0.25)).unwrap_err(),
        SliceError::InvalidInput("Bad min Z value".to_owned())
    );
}
