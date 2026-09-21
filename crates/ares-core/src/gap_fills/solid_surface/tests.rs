use crate::{
    Contour, InfillOptions, Layer, LayerContours, LayerGapFills, Point2,
    bridges::{CounterboreHoleBridging, ExtraBridgeLayer},
    gap_fills::{SolidSurfaceGapFillInput, append_solid_surface_gap_fills},
    options::GapFillTarget,
};

#[test]
fn topbottom_adds_top_and_bottom_gap_fill_but_skips_internal_solid() {
    let gap_fills = generated(GapFillTarget::TopBottom, false, ExtraBridgeLayer::Disabled);

    assert_eq!(gap_fills[0].paths().len(), 1);
    assert!(gap_fills[1].paths().is_empty());
    assert_eq!(gap_fills[2].paths().len(), 1);
}

#[test]
fn everywhere_adds_internal_solid_gap_fill() {
    let gap_fills = generated(GapFillTarget::Everywhere, false, ExtraBridgeLayer::Disabled);

    assert_eq!(gap_fills[1].paths().len(), 1);
}

#[test]
fn nowhere_adds_no_solid_surface_gap_fill() {
    let gap_fills = generated(GapFillTarget::Nowhere, false, ExtraBridgeLayer::Disabled);

    assert!(gap_fills.iter().all(|layer| layer.paths().is_empty()));
}

#[test]
fn spiral_base_topbottom_gap_fill_stops_above_base() {
    let layers = vec![
        rectangle_layer(0, 0.2, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
        rectangle_layer(1, 0.4, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
        rectangle_layer(2, 0.6, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
    ];
    let print_layers = print_layers(&layers);
    let options = InfillOptions::new_for_tests(0.0, 0.0, 0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_spiral_mode_for_tests(true)
        .with_shell_layers_for_tests(2, 0);
    let mut gap_fills = empty_gap_fills(&layers);

    append_solid_surface_gap_fills(
        &mut gap_fills,
        SolidSurfaceGapFillInput {
            print_layers: &print_layers,
            layer_contours: &layers,
            infill_options: &options,
            target: GapFillTarget::TopBottom,
            bridge_no_support: false,
            extra_bridge_layer: ExtraBridgeLayer::Disabled,
            counterbore_hole_bridging: CounterboreHoleBridging::None,
        },
    )
    .unwrap();

    assert_eq!(gap_fills[0].paths().len(), 1);
    assert_eq!(gap_fills[1].paths().len(), 1);
    assert!(gap_fills[2].paths().is_empty());
}

#[test]
fn bridge_no_support_suppresses_bridge_layer_gap_fill() {
    let layers = vec![
        rectangle_layer(0, 0.2, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
        rectangle_layer(1, 0.4, Point2::new(10.0, 0.0), Point2::new(13.0, 0.7)),
    ];
    let gap_fills = generated_for_layers(
        layers,
        GapFillTarget::Everywhere,
        true,
        ExtraBridgeLayer::Disabled,
    );

    assert!(gap_fills[1].paths().is_empty());
}

#[test]
fn uses_solid_line_width_for_threshold_and_inset() {
    let layers = vec![rectangle_layer(
        0,
        0.2,
        Point2::new(0.0, 0.0),
        Point2::new(3.0, 0.7),
    )];
    let gap_fills = generated_for_layers(
        layers,
        GapFillTarget::TopBottom,
        false,
        ExtraBridgeLayer::Disabled,
    );

    assert_eq!(
        gap_fills[0].paths()[0].points(),
        &[Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)]
    );
}

fn generated(
    target: GapFillTarget,
    bridge_no_support: bool,
    extra_bridge_layer: ExtraBridgeLayer,
) -> Vec<LayerGapFills> {
    generated_for_layers(
        vec![
            rectangle_layer(0, 0.2, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
            rectangle_layer(1, 0.4, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
            rectangle_layer(2, 0.6, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
        ],
        target,
        bridge_no_support,
        extra_bridge_layer,
    )
}

fn generated_for_layers(
    layers: Vec<LayerContours>,
    target: GapFillTarget,
    bridge_no_support: bool,
    extra_bridge_layer: ExtraBridgeLayer,
) -> Vec<LayerGapFills> {
    let print_layers = print_layers(&layers);
    let options = options();
    let mut gap_fills = empty_gap_fills(&layers);
    append_solid_surface_gap_fills(
        &mut gap_fills,
        SolidSurfaceGapFillInput {
            print_layers: &print_layers,
            layer_contours: &layers,
            infill_options: &options,
            target,
            bridge_no_support,
            extra_bridge_layer,
            counterbore_hole_bridging: CounterboreHoleBridging::None,
        },
    )
    .unwrap();
    gap_fills
}

fn empty_gap_fills(layers: &[LayerContours]) -> Vec<LayerGapFills> {
    layers
        .iter()
        .map(|layer| LayerGapFills::new(layer.layer_id(), layer.print_z(), Vec::new()))
        .collect()
}

fn print_layers(layers: &[LayerContours]) -> Vec<Layer> {
    layers
        .iter()
        .map(|layer| Layer::new(layer.layer_id(), 0.2, layer.print_z()))
        .collect()
}

fn options() -> InfillOptions {
    InfillOptions::new_for_tests(100.0, 0.0, 0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
}

fn rectangle_layer(layer_id: usize, print_z: f64, min: Point2, max: Point2) -> LayerContours {
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            min,
            Point2::new(max.x(), min.y()),
            max,
            Point2::new(min.x(), max.y()),
        ])],
    )
}
