use crate::{PrintPathRole, SliceOptions, run_slicing_pipeline};
use serde_json::json;

#[test]
fn spiral_mode_normalization_reaches_pipeline_artifacts_and_diagnostics() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 3,
        "line_width": 0.4,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "bottom_shell_layers": 2,
        "top_shell_layers": 3,
        "spiral_mode": true
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(super::square_pyramid_ascii_stl(), &options).unwrap();

    assert_eq!(pipeline.diagnostics().total_perimeter_count(), 2);
    assert!(pipeline.diagnostics().total_infill_count() > 0);
    assert_eq!(
        pipeline
            .layer_perimeters()
            .iter()
            .map(|layer| layer.paths().len())
            .sum::<usize>(),
        2
    );
    assert!(
        pipeline
            .layer_infills()
            .iter()
            .map(|layer| layer.paths().len())
            .sum::<usize>()
            > 0
    );
    assert_eq!(pipeline.options().values()["wall_loops"], json!(1));
    assert_eq!(pipeline.options().values()["top_shell_layers"], json!(0));
    assert_eq!(
        pipeline.options().values()["sparse_infill_density"],
        json!(0)
    );
    assert!(
        pipeline
            .layer_print_paths()
            .iter()
            .flat_map(|layer| layer.paths())
            .any(|path| path.role() == PrintPathRole::ExternalPerimeter)
    );
    assert!(
        pipeline
            .layer_print_paths()
            .iter()
            .flat_map(|layer| layer.paths())
            .any(|path| path.role() == PrintPathRole::BottomSurface)
    );
    assert!(
        pipeline
            .layer_print_paths()
            .iter()
            .flat_map(|layer| layer.paths())
            .any(|path| path.role() == PrintPathRole::TopSolidInfill)
    );
    assert!(
        !pipeline
            .layer_print_paths()
            .iter()
            .skip(2)
            .flat_map(|layer| layer.paths())
            .any(|path| matches!(
                path.role(),
                PrintPathRole::SparseInfill
                    | PrintPathRole::SolidInfill
                    | PrintPathRole::TopSolidInfill
                    | PrintPathRole::BottomSurface
            ))
    );
}
