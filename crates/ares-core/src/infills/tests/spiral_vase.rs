use super::*;
use crate::{
    InfillRole, LayerBrims, LayerGapFills, LayerPerimeters, LayerSkirts, PrintPathInput,
    PrintPathRole, ShellLayerOptions, SliceOptions,
};
use serde_json::json;

#[test]
fn normalized_spiral_mode_generates_bottom_base_before_empty_body() {
    let layers = rectangular_layers(3);
    let options = normalized_spiral_infill_options(2);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(!infills[0].paths().is_empty());
    assert!(!infills[1].paths().is_empty());
    assert!(infills[2].paths().is_empty());
    assert!(
        infills[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::BottomSurface)
    );
    assert!(
        infills[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::TopSurface)
    );
}

#[test]
fn normalized_spiral_mode_maps_base_surfaces_to_print_path_roles() {
    let layers = rectangular_layers(3);
    let print_layers = print_layers(&layers);
    let infills =
        generate_infills(&print_layers, &layers, normalized_spiral_infill_options(2)).unwrap();
    let skirts = empty_skirts(&layers);
    let brims = empty_brims(&layers);
    let perimeters = empty_perimeters(&layers);
    let gap_fills = empty_gap_fills(&layers);

    let print_paths = crate::generate_print_paths(
        PrintPathInput::new(&skirts, &brims, &perimeters, &gap_fills, &infills)
            .with_print_layers(&print_layers),
        ShellLayerOptions::new(2, 0),
        false,
        false,
    )
    .unwrap();

    assert!(
        print_paths[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::BottomSurface)
    );
    assert!(
        print_paths[1]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::TopSolidInfill)
    );
    assert!(print_paths[2].paths().is_empty());
}

#[test]
fn normalized_spiral_mode_short_model_uses_last_existing_base_layer_as_top_surface() {
    let layers = rectangular_layers(2);
    let options = normalized_spiral_infill_options(4);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(
        infills[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::BottomSurface)
    );
    assert!(
        infills[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::TopSurface)
    );
}

#[test]
fn normalized_spiral_mode_without_bottom_layers_keeps_empty_infill() {
    let layers = rectangular_layers(2);
    let options = normalized_spiral_infill_options(0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills.iter().all(|layer| layer.paths().is_empty()));
}

fn normalized_spiral_infill_options(bottom_shell_layers: usize) -> InfillOptions {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": true,
        "sparse_infill_density": 100,
        "sparse_infill_line_width": 0.5,
        "line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": bottom_shell_layers,
        "top_shell_layers": 3,
        "solid_infill_direction": 0,
        "bottom_surface_pattern": "monotonic",
        "top_surface_pattern": "monotonicline"
    }))
    .unwrap();
    options.normalize_fdm(0).unwrap();
    options.infill_options().unwrap()
}

fn rectangular_layers(count: usize) -> Vec<LayerContours> {
    (0..count)
        .map(|index| {
            LayerContours::new(
                index,
                0.2 * (index + 1) as f64,
                square_layer().contours().to_vec(),
            )
        })
        .collect()
}

fn empty_skirts(layers: &[LayerContours]) -> Vec<LayerSkirts> {
    layers
        .iter()
        .map(|layer| LayerSkirts::new(layer.layer_id(), layer.print_z(), Vec::new()))
        .collect()
}

fn empty_brims(layers: &[LayerContours]) -> Vec<LayerBrims> {
    layers
        .iter()
        .map(|layer| LayerBrims::new(layer.layer_id(), layer.print_z(), Vec::new()))
        .collect()
}

fn empty_perimeters(layers: &[LayerContours]) -> Vec<LayerPerimeters> {
    layers
        .iter()
        .map(|layer| LayerPerimeters::new(layer.layer_id(), layer.print_z(), Vec::new()))
        .collect()
}

fn empty_gap_fills(layers: &[LayerContours]) -> Vec<LayerGapFills> {
    layers
        .iter()
        .map(|layer| LayerGapFills::new(layer.layer_id(), layer.print_z(), Vec::new()))
        .collect()
}
