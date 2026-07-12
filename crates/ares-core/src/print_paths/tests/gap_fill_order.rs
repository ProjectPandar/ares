use super::*;
use crate::{GapFillPath, LayerGapFills};

#[test]
fn orders_gap_fill_between_perimeters_and_infills() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(0, 0.2),
            &sample_brims(0, 0.2),
            &sample_perimeters(0, 0.2),
            &non_empty_gap_fills(0, 0.2),
            &sample_infills(0, 0.2),
        ),
        ShellLayerOptions::new(1, 1),
        false,
        false,
    )
    .unwrap();

    assert_eq!(
        output[0]
            .paths()
            .iter()
            .map(PrintPath::role)
            .collect::<Vec<_>>(),
        vec![
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::GapFill,
            PrintPathRole::SparseInfill,
        ]
    );
}

#[test]
fn keeps_gap_fill_after_perimeters_when_infill_first_is_enabled_after_first_layer() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(1, 0.4),
            &sample_brims(1, 0.4),
            &sample_perimeters(1, 0.4),
            &non_empty_gap_fills(1, 0.4),
            &sample_infills(1, 0.4),
        ),
        ShellLayerOptions::new(1, 1),
        true,
        false,
    )
    .unwrap();

    assert_eq!(
        output[0]
            .paths()
            .iter()
            .map(PrintPath::role)
            .collect::<Vec<_>>(),
        vec![
            PrintPathRole::SparseInfill,
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::GapFill,
        ]
    );
}

fn non_empty_gap_fills(layer_id: usize, print_z: f64) -> [LayerGapFills; 1] {
    [LayerGapFills::new(
        layer_id,
        print_z,
        vec![GapFillPath::new(vec![Point2::new(0.4, 0.5), Point2::new(1.6, 0.5)]).unwrap()],
    )]
}
