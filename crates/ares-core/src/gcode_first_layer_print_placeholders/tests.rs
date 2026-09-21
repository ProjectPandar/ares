use super::*;
use crate::{PrintPath, PrintPathRole};

#[test]
fn empty_first_layer_paths_render_empty_placeholder_strings() {
    let rendered = placeholders(&[]);

    assert_eq!(rendered.min_list(), "");
    assert_eq!(rendered.max_list(), "");
    assert_eq!(rendered.size_list(), "");
    assert_eq!(rendered.center_list(), "");
    assert_eq!(rendered.bounds(), None);

    let empty_layer = LayerPrintPaths::new(0, 0.2, Vec::new());
    let rendered = placeholders(&[empty_layer]);

    assert_eq!(rendered.min_list(), "");
    assert_eq!(rendered.max_list(), "");
    assert_eq!(rendered.size_list(), "");
    assert_eq!(rendered.center_list(), "");
    assert_eq!(rendered.bounds(), None);
}

#[test]
fn asymmetric_first_layer_paths_render_center_from_bounds() {
    let layer = LayerPrintPaths::new(
        0,
        0.2,
        vec![
            PrintPath::new(
                PrintPathRole::ExternalPerimeter,
                vec![
                    Point2::new(1.0, -3.0),
                    Point2::new(7.0, 9.0),
                    Point2::new(3.0, 4.0),
                ],
            )
            .unwrap(),
        ],
    );

    let rendered = placeholders(&[layer]);

    assert_eq!(rendered.min_list(), "1,-3");
    assert_eq!(rendered.max_list(), "7,9");
    assert_eq!(rendered.size_list(), "6,12");
    assert_eq!(rendered.center_list(), "4,3");
    assert_eq!(
        rendered.bounds(),
        Some(FirstLayerPrintBounds::new(
            Point2::new(1.0, -3.0),
            Point2::new(7.0, 9.0),
        ))
    );
}
