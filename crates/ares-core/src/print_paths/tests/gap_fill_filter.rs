use super::super::*;
use crate::Point2;

#[test]
fn filters_only_gap_fill_paths_shorter_than_threshold() {
    let layer = LayerPrintPaths::new(
        7,
        1.4,
        vec![
            PrintPath::new(
                PrintPathRole::GapFill,
                vec![Point2::new(0.0, 0.0), Point2::new(0.5, 0.0)],
            )
            .unwrap(),
            PrintPath::new(
                PrintPathRole::GapFill,
                vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)],
            )
            .unwrap(),
            PrintPath::new(
                PrintPathRole::ExternalPerimeter,
                vec![Point2::new(0.0, 0.0), Point2::new(0.25, 0.0)],
            )
            .unwrap(),
        ],
    );

    let filtered = filter_short_gap_fill_paths(vec![layer], 1.0);

    assert_eq!(filtered[0].layer_id(), 7);
    assert_eq!(filtered[0].print_z(), 1.4);
    assert_eq!(filtered[0].paths().len(), 2);
    assert_eq!(filtered[0].paths()[0].role(), PrintPathRole::GapFill);
    assert_eq!(
        filtered[0].paths()[0].points(),
        &[Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)]
    );
    assert_eq!(
        filtered[0].paths()[1].role(),
        PrintPathRole::ExternalPerimeter
    );
}

#[test]
fn default_and_negative_gap_fill_thresholds_keep_constructed_paths() {
    let layer = LayerPrintPaths::new(
        0,
        0.2,
        vec![
            PrintPath::new(
                PrintPathRole::GapFill,
                vec![Point2::new(0.0, 0.0), Point2::new(0.5, 0.0)],
            )
            .unwrap(),
        ],
    );

    assert_eq!(
        filter_short_gap_fill_paths(vec![layer.clone()], 0.0),
        vec![layer.clone()]
    );
    assert_eq!(
        filter_short_gap_fill_paths(vec![layer.clone()], -1.0),
        vec![layer]
    );
}
