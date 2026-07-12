use super::*;
use crate::{ExtrusionRole, InfillRole, PrintPathRole};
use serde_json::json;

#[test]
fn density_100_reaches_pipeline_and_gcode_as_internal_solid_infill() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 100,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "rectilinear",
        "internal_solid_infill_pattern": "grid",
        "solid_infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0
    }))
    .unwrap();

    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(
        pipeline
            .layer_infills()
            .iter()
            .flat_map(|layer| layer.paths())
            .all(|path| path.role() == InfillRole::Solid)
    );
    assert!(
        pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::BottomSurface)
    );
    assert!(
        pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::SolidInfill)
    );
    assert!(
        pipeline.layer_print_paths()[2]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::TopSolidInfill)
    );
    assert!(
        pipeline.print().objects()[0].layers()[1].regions()[0]
            .fills()
            .paths()
            .iter()
            .any(|path| path.role() == ExtrusionRole::SolidInfill)
    );
    assert!(
        pipeline.layer_toolpath_moves()[1]
            .moves()
            .iter()
            .any(|move_| move_.role() == PrintPathRole::SolidInfill)
    );
    assert!(
        pipeline.layer_extrusion_moves()[1]
            .moves()
            .iter()
            .any(|move_| move_.role() == PrintPathRole::SolidInfill)
    );
    assert!(
        pipeline.layer_speed_moves()[1]
            .moves()
            .iter()
            .any(|move_| move_.role() == PrintPathRole::SolidInfill)
    );
    assert!(gcode.contains(";INFILL:solid:"));
    assert!(gcode.contains(";PRINT_PATH:bottom_surface:"));
    assert!(gcode.contains(";PRINT_PATH:solid_infill:"));
    assert!(gcode.contains(";PRINT_PATH:top_solid_infill:"));
    assert!(gcode.contains(";SPEED:print:solid_infill:"));
    assert!(gcode.contains(";EXTRUSION:print:solid_infill:"));
}

#[test]
fn spiral_mode_density_100_generates_bottom_base_infill() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 3,
        "line_width": 0.4,
        "sparse_infill_density": 100,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "bottom_shell_layers": 2,
        "top_shell_layers": 3,
        "spiral_mode": true
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(super::square_pyramid_ascii_stl(), &options).unwrap();

    assert_eq!(
        pipeline.options().values()["sparse_infill_density"],
        json!(0)
    );
    assert_eq!(pipeline.options().values()["top_shell_layers"], json!(0));
    assert!(
        pipeline.layer_infills()[0]
            .paths()
            .iter()
            .any(|path| path.role() == InfillRole::BottomSurface)
    );
    assert!(
        pipeline.layer_infills()[1]
            .paths()
            .iter()
            .any(|path| path.role() == InfillRole::TopSurface)
    );
    assert!(
        pipeline
            .layer_print_paths()
            .iter()
            .flat_map(|layer| layer.paths())
            .any(|path| path.role() == PrintPathRole::TopSolidInfill)
    );
}

#[test]
fn detect_narrow_internal_solid_infill_changes_interior_geometry_but_keeps_role() {
    let enabled = narrow_internal_options(json!({}));
    let disabled = narrow_internal_options(json!({
        "detect_narrow_internal_solid_infill": false
    }));

    let enabled_pipeline = narrow_internal_pipeline(&enabled);
    let disabled_pipeline = narrow_internal_pipeline(&disabled);
    let enabled_gcode =
        String::from_utf8(crate::gcode::format_gcode(&enabled_pipeline, &enabled).unwrap())
            .unwrap();
    let disabled_gcode =
        String::from_utf8(crate::gcode::format_gcode(&disabled_pipeline, &disabled).unwrap())
            .unwrap();

    assert!(enabled_gcode.contains(";PRINT_PATH:solid_infill:0.2,0.2 -> 3.8,0.2"));
    assert!(enabled_gcode.contains(";PRINT_PATH:solid_infill:3.8,0.2 -> 3.8,0.6"));
    assert!(enabled_gcode.contains(";SPEED:print:solid_infill:"));
    assert!(enabled_gcode.contains(";EXTRUSION:print:solid_infill:"));
    assert!(disabled_gcode.contains(";PRINT_PATH:solid_infill:0,0.2 -> 4,0.2"));
    assert_ne!(enabled_gcode, disabled_gcode);
}

fn narrow_internal_options(overrides: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 100,
        "sparse_infill_line_width": 0.4,
        "line_width": 0.4,
        "minimum_sparse_infill_area": 0,
        "internal_solid_infill_pattern": "grid",
        "solid_infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0
    });
    value.as_object_mut().unwrap().extend(
        overrides
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    serde_json::from_value(value).unwrap()
}

fn narrow_internal_contours_by_layer() -> Vec<Vec<crate::Contour>> {
    (0..3)
        .map(|_| {
            vec![crate::Contour::new(vec![
                crate::Point2::new(0.0, 0.0),
                crate::Point2::new(4.0, 0.0),
                crate::Point2::new(4.0, 0.8),
                crate::Point2::new(0.0, 0.8),
            ])]
        })
        .collect()
}

fn narrow_internal_pipeline(options: &SliceOptions) -> SlicingPipeline {
    let layers = (0..3)
        .map(|id| crate::Layer::new(id, 0.2, 0.2 * (id + 1) as f64))
        .collect::<Vec<_>>();
    let layer_slices = layers
        .iter()
        .map(|layer| crate::LayerSlice::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_contours = layers
        .iter()
        .zip(narrow_internal_contours_by_layer())
        .map(|(layer, contours)| crate::LayerContours::new(layer.id(), layer.print_z(), contours))
        .collect::<Vec<_>>();
    let layer_perimeters =
        crate::generate_perimeters(&layer_contours, options.perimeter_options().unwrap()).unwrap();
    let layer_gap_fills = crate::generate_gap_fills(
        &layer_contours,
        options.perimeter_options().unwrap(),
        options
            .speed_options()
            .unwrap()
            .speed_for_role(crate::ToolpathMoveKind::Print, PrintPathRole::GapFill),
    )
    .unwrap();
    let layer_infills =
        crate::generate_infills(&layers, &layer_contours, options.infill_options().unwrap())
            .unwrap();
    let layer_skirts = layers
        .iter()
        .map(|layer| crate::LayerSkirts::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_brims = layers
        .iter()
        .map(|layer| crate::LayerBrims::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_print_paths = crate::generate_print_paths(
        crate::PrintPathInput::new(
            &layer_skirts,
            &layer_brims,
            &layer_perimeters,
            &layer_gap_fills,
            &layer_infills,
        )
        .with_layer_contours(&layer_contours)
        .with_print_layers(&layers),
        options.shell_layer_options().unwrap(),
        false,
        false,
    )
    .unwrap();
    let print = crate::build_print_domain(&layers, &layer_contours, &layer_print_paths).unwrap();
    let layer_toolpath_moves = crate::generate_toolpath_moves(&layer_print_paths);
    let extrusion_options = options.extrusion_options().unwrap();
    let layer_extrusion_moves =
        crate::generate_extrusion_moves(&layers, &layer_toolpath_moves, extrusion_options).unwrap();
    let layer_speed_moves =
        crate::generate_speed_moves(&layer_extrusion_moves, options.speed_options().unwrap());
    let total_extrusion_mm = layer_extrusion_moves
        .iter()
        .map(|layer| layer.total_extrusion_mm())
        .sum();

    SlicingPipeline {
        options: options.clone(),
        model: crate::Model::new(InputFormat::Stl, Vec::new()),
        layers,
        layer_slices,
        layer_contours,
        layer_perimeters,
        layer_gap_fills,
        layer_infills,
        layer_skirts,
        layer_brims,
        layer_print_paths,
        print,
        layer_toolpath_moves,
        layer_extrusion_moves,
        layer_speed_moves,
        diagnostics: PipelineDiagnostics {
            completed_stages: vec![
                PipelineStage::Model,
                PipelineStage::Layers,
                PipelineStage::Segments,
                PipelineStage::Contours,
                PipelineStage::Perimeters,
                PipelineStage::Infills,
                PipelineStage::Skirts,
                PipelineStage::Brims,
                PipelineStage::PrintPaths,
                PipelineStage::Moves,
                PipelineStage::Extrusions,
                PipelineStage::Speeds,
            ],
            input_format: InputFormat::Stl,
            triangle_count: 0,
            layer_count: 3,
            total_segment_count: 0,
            total_contour_count: 3,
            total_perimeter_count: 0,
            total_infill_count: 0,
            total_skirt_path_count: 0,
            total_brim_path_count: 0,
            total_print_path_count: 0,
            total_toolpath_move_count: 0,
            total_extrusion_move_count: 0,
            total_speed_move_count: 0,
            total_extrusion_mm,
            empty_layer_count: 0,
            option_count: options.values().len(),
        },
    }
}
