use super::*;

#[test]
fn maps_shell_layer_counts_to_bottom_interior_and_top_surface_roles() {
    let output = solid_layers_output(5, ShellLayerOptions::new(2, 1));

    assert_eq!(
        solid_roles(&output),
        vec![
            PrintPathRole::BottomSurface,
            PrintPathRole::BottomSurface,
            PrintPathRole::SolidInfill,
            PrintPathRole::SolidInfill,
            PrintPathRole::TopSolidInfill,
        ]
    );
}

#[test]
fn zero_bottom_shell_layers_disable_bottom_surface_role() {
    let output = solid_layers_output(4, ShellLayerOptions::new(0, 2));

    assert_eq!(
        solid_roles(&output),
        vec![
            PrintPathRole::SolidInfill,
            PrintPathRole::SolidInfill,
            PrintPathRole::TopSolidInfill,
            PrintPathRole::TopSolidInfill,
        ]
    );
}

#[test]
fn zero_top_shell_layers_disable_top_surface_role() {
    let output = solid_layers_output(4, ShellLayerOptions::new(2, 0));

    assert_eq!(
        solid_roles(&output),
        vec![
            PrintPathRole::BottomSurface,
            PrintPathRole::BottomSurface,
            PrintPathRole::SolidInfill,
            PrintPathRole::SolidInfill,
        ]
    );
}

#[test]
fn overlapping_shell_layer_ranges_prefer_bottom_surface_role() {
    let output = solid_layers_output(5, ShellLayerOptions::new(3, 4));

    assert_eq!(
        solid_roles(&output),
        vec![
            PrintPathRole::BottomSurface,
            PrintPathRole::BottomSurface,
            PrintPathRole::BottomSurface,
            PrintPathRole::TopSolidInfill,
            PrintPathRole::TopSolidInfill,
        ]
    );
    assert_eq!(PrintPathRole::BottomSurface.as_str(), "bottom_surface");
    assert_eq!(PrintPathRole::TopSolidInfill.as_str(), "top_solid_infill");
}

fn solid_layers_output(
    layer_count: usize,
    shell_layers: ShellLayerOptions,
) -> Vec<LayerPrintPaths> {
    let layers = (0..layer_count)
        .map(|layer_id| (layer_id, 0.2 * (layer_id + 1) as f64))
        .collect::<Vec<_>>();
    let skirts = layers
        .iter()
        .map(|(layer_id, print_z)| LayerSkirts::new(*layer_id, *print_z, Vec::new()))
        .collect::<Vec<_>>();
    let brims = layers
        .iter()
        .map(|(layer_id, print_z)| LayerBrims::new(*layer_id, *print_z, Vec::new()))
        .collect::<Vec<_>>();
    let perimeters = layers
        .iter()
        .map(|(layer_id, print_z)| {
            sample_perimeters(*layer_id, *print_z)
                .into_iter()
                .next()
                .unwrap()
        })
        .collect::<Vec<_>>();
    let gap_fills = layers
        .iter()
        .map(|(layer_id, print_z)| LayerGapFills::new(*layer_id, *print_z, Vec::new()))
        .collect::<Vec<_>>();
    let infills = layers
        .iter()
        .map(|(layer_id, print_z)| solid_infill_layer(*layer_id, *print_z))
        .collect::<Vec<_>>();

    generate_print_paths(
        PrintPathInput::new(&skirts, &brims, &perimeters, &gap_fills, &infills),
        shell_layers,
        false,
        false,
    )
    .unwrap()
}

fn solid_roles(output: &[LayerPrintPaths]) -> Vec<PrintPathRole> {
    output
        .iter()
        .map(|layer| {
            layer
                .paths()
                .iter()
                .find(|path| {
                    matches!(
                        path.role(),
                        PrintPathRole::BottomSurface
                            | PrintPathRole::SolidInfill
                            | PrintPathRole::TopSolidInfill
                    )
                })
                .unwrap()
                .role()
        })
        .collect()
}

#[test]
fn single_layer_solid_infill_prefers_bottom_surface_role() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(0, 0.2),
            &sample_brims(0, 0.2),
            &sample_perimeters(0, 0.2),
            &sample_gap_fills(0, 0.2),
            &[solid_infill_layer(0, 0.2)],
        ),
        ShellLayerOptions::new(1, 1),
        false,
        false,
    )
    .unwrap();

    assert!(
        output[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::BottomSurface)
    );
    assert!(
        !output[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::TopSolidInfill)
    );
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
