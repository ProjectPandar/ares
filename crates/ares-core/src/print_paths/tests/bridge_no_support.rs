use super::*;
use crate::{
    Contour, LayerContours, PerimeterPath, PerimeterRole,
    bridges::{BridgeLayerPolicy, CounterboreHoleBridging, ExtraBridgeLayer},
};

#[test]
fn bridge_no_support_true_maps_unsupported_bottom_solid_to_bridge() {
    let output = output_for_layers(true, unsupported_layers());

    assert_eq!(solid_role(&output[1]), PrintPathRole::Bridge);
}

#[test]
fn bridge_no_support_false_keeps_unsupported_bottom_solid_as_bottom_surface() {
    let output = output_for_layers(false, unsupported_layers());

    assert_eq!(solid_role(&output[1]), PrintPathRole::BottomSurface);
}

#[test]
fn bridge_no_support_true_keeps_supported_bottom_solid_as_bottom_surface() {
    let output = output_for_layers(true, supported_layers());

    assert_eq!(solid_role(&output[1]), PrintPathRole::BottomSurface);
}

#[test]
fn bridge_no_support_true_keeps_first_layer_bottom_solid_as_bottom_surface() {
    let output = output_for_layers(true, vec![rectangle_layer(0, 0.2, (0.0, 0.0, 4.0, 4.0))]);

    assert_eq!(solid_role(&output[0]), PrintPathRole::BottomSurface);
}

#[test]
fn extra_external_bridge_layer_maps_next_bottom_solid_to_bridge() {
    let output = output_for_extra_external_bridge_layer(true, ExtraBridgeLayer::ExternalBridgeOnly);

    assert_eq!(solid_role(&output[1]), PrintPathRole::Bridge);
    assert_eq!(solid_role(&output[2]), PrintPathRole::Bridge);
}

#[test]
fn internal_only_extra_bridge_layer_does_not_map_external_next_layer() {
    let output = output_for_extra_external_bridge_layer(true, ExtraBridgeLayer::InternalBridgeOnly);

    assert_eq!(solid_role(&output[1]), PrintPathRole::Bridge);
    assert_eq!(solid_role(&output[2]), PrintPathRole::BottomSurface);
}

#[test]
fn sacrificial_counterbore_keeps_unsupported_bottom_solid_as_bottom_surface() {
    let output = output_for_layers_with_extra_bridge_layer(
        true,
        ExtraBridgeLayer::Disabled,
        CounterboreHoleBridging::SacrificialLayer,
        unsupported_layers(),
        ShellLayerOptions::new(2, 0),
    );

    assert_eq!(solid_role(&output[1]), PrintPathRole::BottomSurface);
}

fn output_for_layers(
    bridge_no_support: bool,
    layer_contours: Vec<LayerContours>,
) -> Vec<LayerPrintPaths> {
    output_for_layers_with_extra_bridge_layer(
        bridge_no_support,
        ExtraBridgeLayer::Disabled,
        CounterboreHoleBridging::None,
        layer_contours,
        ShellLayerOptions::new(2, 0),
    )
}

fn output_for_extra_external_bridge_layer(
    bridge_no_support: bool,
    extra_bridge_layer: ExtraBridgeLayer,
) -> Vec<LayerPrintPaths> {
    output_for_layers_with_extra_bridge_layer(
        bridge_no_support,
        extra_bridge_layer,
        CounterboreHoleBridging::None,
        vec![
            rectangle_layer(0, 0.2, (0.0, 0.0, 4.0, 4.0)),
            rectangle_layer(1, 0.4, (10.0, 0.0, 14.0, 4.0)),
            rectangle_layer(2, 0.6, (10.0, 0.0, 14.0, 4.0)),
        ],
        ShellLayerOptions::new(3, 0),
    )
}

fn output_for_layers_with_extra_bridge_layer(
    bridge_no_support: bool,
    extra_bridge_layer: ExtraBridgeLayer,
    counterbore_hole_bridging: CounterboreHoleBridging,
    layer_contours: Vec<LayerContours>,
    shell_layers: ShellLayerOptions,
) -> Vec<LayerPrintPaths> {
    let skirts = layer_contours
        .iter()
        .map(|layer| LayerSkirts::new(layer.layer_id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let brims = layer_contours
        .iter()
        .map(|layer| LayerBrims::new(layer.layer_id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let perimeters = layer_contours
        .iter()
        .map(|layer| {
            LayerPerimeters::new(
                layer.layer_id(),
                layer.print_z(),
                vec![
                    PerimeterPath::new(
                        PerimeterRole::External,
                        layer.contours()[0].points().to_vec(),
                    )
                    .unwrap(),
                ],
            )
        })
        .collect::<Vec<_>>();
    let gap_fills = layer_contours
        .iter()
        .map(|layer| LayerGapFills::new(layer.layer_id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let infills = layer_contours
        .iter()
        .map(|layer| {
            LayerInfills::new(
                layer.layer_id(),
                layer.print_z(),
                vec![
                    InfillPath::new(
                        InfillRole::Solid,
                        vec![Point2::new(0.5, 0.0), Point2::new(0.5, 1.0)],
                        0.2,
                    )
                    .unwrap(),
                ],
            )
        })
        .collect::<Vec<_>>();

    crate::generate_print_paths_with_bridge_policy(
        PrintPathInput::new(&skirts, &brims, &perimeters, &gap_fills, &infills)
            .with_layer_contours(&layer_contours),
        shell_layers,
        false,
        BridgeLayerPolicy::new(
            bridge_no_support,
            extra_bridge_layer,
            counterbore_hole_bridging,
        ),
    )
    .unwrap()
}

fn unsupported_layers() -> Vec<LayerContours> {
    vec![
        rectangle_layer(0, 0.2, (0.0, 0.0, 4.0, 4.0)),
        rectangle_layer(1, 0.4, (10.0, 0.0, 14.0, 4.0)),
    ]
}

fn supported_layers() -> Vec<LayerContours> {
    vec![
        rectangle_layer(0, 0.2, (0.0, 0.0, 4.0, 4.0)),
        rectangle_layer(1, 0.4, (0.0, 0.0, 4.0, 4.0)),
    ]
}

fn rectangle_layer(layer_id: usize, print_z: f64, bounds: (f64, f64, f64, f64)) -> LayerContours {
    let (min_x, min_y, max_x, max_y) = bounds;
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(min_x, min_y),
            Point2::new(max_x, min_y),
            Point2::new(max_x, max_y),
            Point2::new(min_x, max_y),
        ])],
    )
}

fn solid_role(layer: &LayerPrintPaths) -> PrintPathRole {
    layer
        .paths()
        .iter()
        .find(|path| {
            matches!(
                path.role(),
                PrintPathRole::Bridge
                    | PrintPathRole::BottomSurface
                    | PrintPathRole::SolidInfill
                    | PrintPathRole::TopSolidInfill
            )
        })
        .unwrap()
        .role()
}
