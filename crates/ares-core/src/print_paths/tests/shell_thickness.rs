use super::*;
use crate::{
    Contour, InfillPath, InfillRole, Layer, LayerBrims, LayerContours, LayerGapFills,
    LayerPerimeters, LayerSkirts, PerimeterPath, PerimeterRole, SliceError,
};

fn layers(heights: &[f64]) -> Vec<Layer> {
    let mut print_z = 0.0;
    heights
        .iter()
        .enumerate()
        .map(|(id, height)| {
            print_z += *height;
            Layer::new(id, *height, print_z)
        })
        .collect()
}

fn solid_infill_layer(layer_id: usize, print_z: f64) -> LayerInfills {
    LayerInfills::new(
        layer_id,
        print_z,
        vec![
            InfillPath::new(
                InfillRole::Solid,
                vec![Point2::new(0.5, 0.0), Point2::new(0.5, 1.0)],
                0.2,
            )
            .unwrap(),
        ],
    )
}

#[test]
fn bottom_thickness_expands_with_orca_bottom_z_window() {
    let layers = layers(&[0.2, 0.2, 0.2, 0.2]);
    let shell = ShellLayerOptions::with_thicknesses(1, 0.45, 0, 0.0);

    assert!(shell.is_bottom_shell(&layers, 0));
    assert!(shell.is_bottom_shell(&layers, 1));
    assert!(shell.is_bottom_shell(&layers, 2));
    assert!(!shell.is_bottom_shell(&layers, 3));
}

#[test]
fn top_thickness_expands_with_orca_print_z_window() {
    let layers = layers(&[0.2, 0.2, 0.2, 0.2]);
    let shell = ShellLayerOptions::with_thicknesses(0, 0.0, 1, 0.45);

    assert!(!shell.is_top_shell(&layers, 0));
    assert!(shell.is_top_shell(&layers, 1));
    assert!(shell.is_top_shell(&layers, 2));
    assert!(shell.is_top_shell(&layers, 3));
}

#[test]
fn zero_layer_count_disables_positive_thickness() {
    let layers = layers(&[0.2, 0.2, 0.2]);
    let shell = ShellLayerOptions::with_thicknesses(0, 1.0, 0, 1.0);

    assert!(!shell.is_bottom_shell(&layers, 0));
    assert!(!shell.is_top_shell(&layers, 2));
}

#[test]
fn shell_layer_counts_remain_minimums_when_thickness_is_smaller() {
    let layers = layers(&[0.2, 0.2, 0.2, 0.2, 0.2]);
    let bottom_shell = ShellLayerOptions::with_thicknesses(3, 0.1, 0, 0.0);
    let top_shell = ShellLayerOptions::with_thicknesses(0, 0.0, 3, 0.1);

    assert!(bottom_shell.is_bottom_shell(&layers, 0));
    assert!(bottom_shell.is_bottom_shell(&layers, 1));
    assert!(bottom_shell.is_bottom_shell(&layers, 2));
    assert!(!bottom_shell.is_bottom_shell(&layers, 3));
    assert!(!top_shell.is_top_shell(&layers, 1));
    assert!(top_shell.is_top_shell(&layers, 2));
    assert!(top_shell.is_top_shell(&layers, 3));
    assert!(top_shell.is_top_shell(&layers, 4));
}

#[test]
fn variable_layer_heights_use_layer_z_boundaries() {
    let bottom_layers = layers(&[0.3, 0.1, 0.25, 0.2]);
    let top_layers = layers(&[0.2, 0.25, 0.1, 0.3]);
    let bottom_shell = ShellLayerOptions::with_thicknesses(1, 0.45, 0, 0.0);
    let top_shell = ShellLayerOptions::with_thicknesses(0, 0.0, 1, 0.45);

    assert!(bottom_shell.is_bottom_shell(&bottom_layers, 2));
    assert!(!bottom_shell.is_bottom_shell(&bottom_layers, 3));
    assert!(top_shell.is_top_shell(&top_layers, 1));
    assert!(!top_shell.is_top_shell(&top_layers, 0));
}

#[test]
fn thickness_boundary_is_strict_with_epsilon() {
    let exact = layers(&[0.2, 0.2]);
    let bottom_below = vec![Layer::new(0, 0.2, 0.2), Layer::new(1, 0.2, 0.399_998)];
    let bottom_above = vec![Layer::new(0, 0.2, 0.2), Layer::new(1, 0.2, 0.400_002)];
    let top_below = vec![Layer::new(0, 0.2, 0.200_002), Layer::new(1, 0.2, 0.4)];
    let top_above = vec![Layer::new(0, 0.2, 0.199_998), Layer::new(1, 0.2, 0.4)];
    let shell = ShellLayerOptions::with_thicknesses(1, 0.2, 1, 0.2);

    assert!(!shell.is_bottom_shell(&exact, 1));
    assert!(shell.is_bottom_shell(&bottom_below, 1));
    assert!(!shell.is_bottom_shell(&bottom_above, 1));
    assert!(!shell.is_top_shell(&exact, 0));
    assert!(shell.is_top_shell(&top_below, 0));
    assert!(!shell.is_top_shell(&top_above, 0));
}

#[test]
fn bottom_precedes_top_when_shell_ranges_overlap() {
    let layers = layers(&[0.2, 0.2, 0.2]);
    let shell = ShellLayerOptions::with_thicknesses(3, 1.0, 3, 1.0);

    assert_eq!(
        shell.solid_role(&layers, 1, false),
        PrintPathRole::BottomSurface
    );
}

fn output_with_print_layers(
    print_layers: &[Layer],
    shell: ShellLayerOptions,
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(0, 0.2),
            &sample_brims(0, 0.2),
            &sample_perimeters(0, 0.2),
            &sample_gap_fills(0, 0.2),
            &[solid_infill_layer(0, 0.2)],
        )
        .with_print_layers(print_layers),
        shell,
        false,
        false,
    )
}

fn solid_output_with_print_layers(
    print_layers: &[Layer],
    shell: ShellLayerOptions,
) -> Vec<LayerPrintPaths> {
    let skirts = print_layers
        .iter()
        .map(|layer| LayerSkirts::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let brims = print_layers
        .iter()
        .map(|layer| LayerBrims::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let perimeters = print_layers
        .iter()
        .map(|layer| {
            sample_perimeters(layer.id(), layer.print_z())
                .into_iter()
                .next()
                .unwrap()
        })
        .collect::<Vec<_>>();
    let gap_fills = print_layers
        .iter()
        .map(|layer| LayerGapFills::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let infills = print_layers
        .iter()
        .map(|layer| solid_infill_layer(layer.id(), layer.print_z()))
        .collect::<Vec<_>>();

    generate_print_paths(
        PrintPathInput::new(&skirts, &brims, &perimeters, &gap_fills, &infills)
            .with_print_layers(print_layers),
        shell,
        false,
        false,
    )
    .unwrap()
}

fn two_layer_metadata_with_print_layers(
    print_layers: &[Layer],
    shell: ShellLayerOptions,
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    let skirts = vec![
        LayerSkirts::new(0, 0.2, Vec::new()),
        LayerSkirts::new(1, 0.4, Vec::new()),
    ];
    let brims = vec![
        LayerBrims::new(0, 0.2, Vec::new()),
        LayerBrims::new(1, 0.4, Vec::new()),
    ];
    let perimeters = [sample_perimeters(0, 0.2), sample_perimeters(1, 0.4)]
        .into_iter()
        .map(|mut layers| layers.remove(0))
        .collect::<Vec<_>>();
    let gap_fills = vec![
        LayerGapFills::new(0, 0.2, Vec::new()),
        LayerGapFills::new(1, 0.4, Vec::new()),
    ];
    let infills = vec![solid_infill_layer(0, 0.2), solid_infill_layer(1, 0.4)];

    generate_print_paths(
        PrintPathInput::new(&skirts, &brims, &perimeters, &gap_fills, &infills)
            .with_print_layers(print_layers),
        shell,
        false,
        false,
    )
}

#[test]
fn print_path_input_rejects_mismatched_print_layers() {
    let shell = ShellLayerOptions::new(1, 0);

    assert!(output_with_print_layers(&[], shell).is_err());
    assert!(output_with_print_layers(&[Layer::new(1, 0.2, 0.2)], shell).is_err());
    assert!(output_with_print_layers(&[Layer::new(0, 0.2, 0.3)], shell).is_err());
    assert!(
        two_layer_metadata_with_print_layers(
            &[Layer::new(0, 0.2, 0.2), Layer::new(2, 0.2, 0.4)],
            shell,
        )
        .is_err()
    );
    assert!(
        two_layer_metadata_with_print_layers(
            &[Layer::new(0, 0.2, 0.2), Layer::new(1, 0.2, 0.5)],
            shell,
        )
        .is_err()
    );
}

#[test]
fn print_path_input_rejects_invalid_print_layer_geometry() {
    let shell = ShellLayerOptions::new(1, 0);

    assert!(output_with_print_layers(&[Layer::new(0, f64::NAN, 0.2)], shell).is_err());
    assert!(output_with_print_layers(&[Layer::new(0, f64::INFINITY, 0.2)], shell).is_err());
    assert!(output_with_print_layers(&[Layer::new(0, 0.0, 0.2)], shell).is_err());
    assert!(output_with_print_layers(&[Layer::new(0, -0.1, 0.2)], shell).is_err());
    assert!(output_with_print_layers(&[Layer::new(0, 0.2, f64::INFINITY)], shell).is_err());
    assert!(output_with_print_layers(&[Layer::new(0, 0.2, f64::NAN)], shell).is_err());
}

#[test]
fn omitting_print_layers_keeps_count_only_classification() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(0, 0.2),
            &sample_brims(0, 0.2),
            &sample_perimeters(0, 0.2),
            &sample_gap_fills(0, 0.2),
            &[solid_infill_layer(0, 0.2)],
        ),
        ShellLayerOptions::with_thicknesses(0, 1.0, 0, 1.0),
        false,
        false,
    )
    .unwrap();

    assert_eq!(output[0].paths()[1].role(), PrintPathRole::SolidInfill);
}

#[test]
fn print_layers_enable_bottom_thickness_classification() {
    let layers = layers(&[0.2, 0.2, 0.2, 0.2]);
    let shell = ShellLayerOptions::with_thicknesses(1, 0.45, 0, 0.0);
    let output = solid_output_with_print_layers(&layers, shell);

    assert_eq!(output[2].paths()[1].role(), PrintPathRole::BottomSurface);
}

#[test]
fn print_layers_enable_top_thickness_classification() {
    let layers = layers(&[0.2, 0.2, 0.2, 0.2]);
    let shell = ShellLayerOptions::with_thicknesses(0, 0.0, 1, 0.45);
    let output = solid_output_with_print_layers(&layers, shell);

    assert_eq!(output[1].paths()[1].role(), PrintPathRole::TopSolidInfill);
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

fn solid_output_with_print_layers_and_contours(
    print_layers: &[Layer],
    layer_contours: &[LayerContours],
    shell: ShellLayerOptions,
    bridge_no_support: bool,
) -> Vec<LayerPrintPaths> {
    let skirts = print_layers
        .iter()
        .map(|layer| LayerSkirts::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let brims = print_layers
        .iter()
        .map(|layer| LayerBrims::new(layer.id(), layer.print_z(), Vec::new()))
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
    let gap_fills = print_layers
        .iter()
        .map(|layer| LayerGapFills::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let infills = print_layers
        .iter()
        .map(|layer| solid_infill_layer(layer.id(), layer.print_z()))
        .collect::<Vec<_>>();

    generate_print_paths(
        PrintPathInput::new(&skirts, &brims, &perimeters, &gap_fills, &infills)
            .with_layer_contours(layer_contours)
            .with_print_layers(print_layers),
        shell,
        false,
        bridge_no_support,
    )
    .unwrap()
}

#[test]
fn layer_aware_bridge_no_support_maps_unsupported_bottom_shell_to_bridge() {
    let layers = layers(&[0.2, 0.2]);
    let contours = vec![
        rectangle_layer(0, 0.2, (0.0, 0.0, 4.0, 4.0)),
        rectangle_layer(1, 0.4, (10.0, 0.0, 14.0, 4.0)),
    ];
    let shell = ShellLayerOptions::with_thicknesses(1, 0.45, 0, 0.0);
    let output = solid_output_with_print_layers_and_contours(&layers, &contours, shell, true);

    assert_eq!(output[1].paths()[1].role(), PrintPathRole::Bridge);
}
