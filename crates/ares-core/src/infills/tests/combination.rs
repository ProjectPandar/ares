use super::*;
use crate::Layer;

fn three_square_layers() -> (Vec<Layer>, Vec<LayerContours>) {
    let layers = (0..3)
        .map(|id| Layer::new(id, 0.2, 0.2 * (id + 1) as f64))
        .collect::<Vec<_>>();
    let contours = layers
        .iter()
        .map(|layer| {
            LayerContours::new(
                layer.id(),
                layer.print_z(),
                square_layer().contours().to_vec(),
            )
        })
        .collect::<Vec<_>>();
    (layers, contours)
}

#[test]
fn disabled_infill_combination_preserves_physical_sparse_infill_heights() {
    let (layers, contours) = three_square_layers();

    let infills =
        generate_infills(&layers, &contours, options(InfillPattern::Rectilinear)).unwrap();

    assert_eq!(infills[1].paths().len(), 2);
    assert_eq!(infills[2].paths().len(), 2);
    assert!(
        infills
            .iter()
            .flat_map(LayerInfills::paths)
            .all(|path| path.effective_layer_height_mm() == 0.2)
    );
}

#[test]
fn infill_combination_clears_lower_layer_and_marks_target_height() {
    let (layers, contours) = three_square_layers();
    let options = options(InfillPattern::Rectilinear).with_infill_combination_for_tests(0.4);

    let infills = generate_infills(&layers, &contours, options).unwrap();

    assert_eq!(infills[0].paths().len(), 2);
    assert!(infills[1].paths().is_empty());
    assert_eq!(infills[2].paths().len(), 2);
    assert!(
        infills[0]
            .paths()
            .iter()
            .all(|path| path.effective_layer_height_mm() == 0.2)
    );
    assert!(
        infills[2]
            .paths()
            .iter()
            .all(|path| path.effective_layer_height_mm() == 0.4)
    );
}

#[test]
fn infill_combination_keeps_zero_density_empty() {
    let (layers, contours) = three_square_layers();
    let options =
        InfillOptions::new_for_tests(0.0, 0.0, 0.5).with_infill_combination_for_tests(0.4);

    let infills = generate_infills(&layers, &contours, options).unwrap();

    assert!(infills.iter().all(|layer| layer.paths().is_empty()));
}

#[test]
fn infill_generation_rejects_mismatched_layer_metadata() {
    let (_, contours) = three_square_layers();
    let layers = vec![Layer::new(99, 0.2, 0.2)];

    let result = generate_infills(&layers, &contours, options(InfillPattern::Rectilinear));

    assert!(matches!(
        result,
        Err(SliceError::InvalidInput(message))
            if message.contains("layer and infill contour metadata must match")
    ));
}
