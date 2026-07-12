use super::*;
use crate::{
    Contour, InputFormat, Layer, LayerBrims, LayerContours, LayerGapFills, LayerInfills,
    LayerPerimeters, LayerPrintPaths, LayerSkirts, LayerSlice, Model, Point2, PrintPath,
    PrintPathRole, SliceOptions, SolidSurfaceGapFillInput, ToolpathMoveKind,
    append_solid_surface_gap_fills, build_print_domain, generate_brims, generate_extrusion_moves,
    generate_gap_fills, generate_perimeters, generate_speed_moves, generate_toolpath_moves,
    skirts::generate_skirts_after_brims,
};
pub fn rectangular_pipeline(options: &SliceOptions) -> SlicingPipeline {
    rectangular_layers_pipeline(options, 1)
}

pub fn rectangular_layers_pipeline(options: &SliceOptions, layer_count: usize) -> SlicingPipeline {
    contour_layers_pipeline(
        options,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 4.0),
            Point2::new(0.0, 4.0),
        ])],
        layer_count,
    )
}

pub fn narrow_rectangular_gap_fill_pipeline(options: &SliceOptions) -> SlicingPipeline {
    contours_pipeline(
        options,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(3.0, 0.0),
            Point2::new(3.0, 0.7),
            Point2::new(0.0, 0.7),
        ])],
    )
}

pub fn kinked_brim_pipeline(options: &SliceOptions) -> SlicingPipeline {
    contours_pipeline(
        options,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 4.0),
            Point2::new(2.1, 4.0),
            Point2::new(2.0, 4.2),
            Point2::new(1.9, 4.0),
            Point2::new(0.0, 4.0),
        ])],
    )
}

pub fn contours_pipeline(options: &SliceOptions, contours: Vec<Contour>) -> SlicingPipeline {
    contour_layers_pipeline(options, contours, 1)
}

pub fn unsupported_second_layer_pipeline(options: &SliceOptions) -> SlicingPipeline {
    contour_layers_pipeline_from_layers_for_tests(
        options,
        vec![
            vec![Contour::new(vec![
                Point2::new(0.0, 0.0),
                Point2::new(4.0, 0.0),
                Point2::new(4.0, 4.0),
                Point2::new(0.0, 4.0),
            ])],
            vec![Contour::new(vec![
                Point2::new(10.0, 0.0),
                Point2::new(14.0, 0.0),
                Point2::new(14.0, 4.0),
                Point2::new(10.0, 4.0),
            ])],
        ],
    )
}

pub fn contour_layers_pipeline(
    options: &SliceOptions,
    contours: Vec<Contour>,
    layer_count: usize,
) -> SlicingPipeline {
    contour_layers_pipeline_from_layers_for_tests(options, vec![contours; layer_count])
}

pub(crate) fn contour_layers_pipeline_from_layers_for_tests(
    options: &SliceOptions,
    contours_by_layer: Vec<Vec<Contour>>,
) -> SlicingPipeline {
    let layers = (0..contours_by_layer.len())
        .map(|id| Layer::new(id, 0.2, 0.2 * (id + 1) as f64))
        .collect::<Vec<_>>();
    let layer_slices = layers
        .iter()
        .map(|layer| LayerSlice::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_contours = layers
        .iter()
        .zip(contours_by_layer)
        .map(|(layer, contours)| LayerContours::new(layer.id(), layer.print_z(), contours))
        .collect::<Vec<_>>();
    let layer_contours = crate::make_overhang_printable_contours(
        layer_contours,
        options.perimeter_options().unwrap(),
    );
    let layer_perimeters =
        generate_perimeters(&layer_contours, options.perimeter_options().unwrap()).unwrap();
    let mut layer_gap_fills = generate_gap_fills(
        &layer_contours,
        options.perimeter_options().unwrap(),
        options
            .speed_options()
            .unwrap()
            .speed_for_role(ToolpathMoveKind::Print, PrintPathRole::GapFill),
    )
    .unwrap();
    let bridge_options = options.bridge_options().unwrap();
    let infill_options = options.infill_options().unwrap();
    append_solid_surface_gap_fills(
        &mut layer_gap_fills,
        SolidSurfaceGapFillInput {
            print_layers: &layers,
            layer_contours: &layer_contours,
            infill_options: &infill_options,
            target: options.gap_fill_target().unwrap(),
            bridge_no_support: bridge_options.bridge_no_support(),
            extra_bridge_layer: bridge_options.extra_bridge_layer(),
            counterbore_hole_bridging: bridge_options.counterbore_hole_bridging(),
        },
    )
    .unwrap();
    let layer_infills = crate::infills::generate_infills_with_bridge_context(
        &layers,
        &layer_contours,
        infill_options,
        Some(crate::infills::InfillBridgeContext::new(
            &layer_contours,
            bridge_options.bridge_no_support(),
            bridge_options.extra_bridge_layer(),
            bridge_options.counterbore_hole_bridging(),
        )),
    )
    .unwrap();
    let extrusion_options = options.extrusion_options().unwrap();
    let layer_brims = generate_brims(
        &layer_contours,
        options.brim_options().unwrap(),
        extrusion_options.width_for_role(PrintPathRole::Brim),
    )
    .unwrap();
    let skirt_extrusion_per_mm = extrusion_options
        .extrusion_per_mm(
            PrintPathRole::Skirt,
            options.initial_layer_print_height().unwrap(),
        )
        .unwrap();
    let layer_skirts = generate_skirts_after_brims(
        &layer_contours,
        &layer_brims,
        options.skirt_options().unwrap(),
        extrusion_options.width_for_role(PrintPathRole::Skirt),
        skirt_extrusion_per_mm,
    )
    .unwrap();
    let layer_print_paths = crate::finalize_print_paths_with_layer_contours(
        crate::generate_print_paths_with_bridge_policy(
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
            options.is_infill_first().unwrap(),
            crate::bridges::BridgeLayerPolicy::new(
                bridge_options.bridge_no_support(),
                bridge_options.extra_bridge_layer(),
                bridge_options.counterbore_hole_bridging(),
            ),
        )
        .unwrap(),
        options,
        &layer_contours,
    )
    .unwrap();
    let print = build_print_domain(&layers, &layer_contours, &layer_print_paths).unwrap();
    let layer_toolpath_moves = generate_toolpath_moves(&layer_print_paths);
    let layer_extrusion_moves =
        generate_extrusion_moves(&layers, &layer_toolpath_moves, extrusion_options).unwrap();
    let layer_speed_moves =
        generate_speed_moves(&layer_extrusion_moves, options.speed_options().unwrap());
    let diagnostics = test_diagnostics(
        options,
        TestDiagnosticsInput {
            layers: &layers,
            layer_slices: &layer_slices,
            layer_contours: &layer_contours,
            layer_perimeters: &layer_perimeters,
            layer_infills: &layer_infills,
            layer_skirts: &layer_skirts,
            layer_brims: &layer_brims,
            layer_print_paths: &layer_print_paths,
            layer_toolpath_moves: &layer_toolpath_moves,
            layer_extrusion_moves: &layer_extrusion_moves,
            layer_speed_moves: &layer_speed_moves,
        },
    );

    SlicingPipeline {
        options: options.clone(),
        model: Model::new(InputFormat::Stl, Vec::new()),
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
        diagnostics,
    }
}

pub fn single_path_pipeline(
    options: &SliceOptions,
    role: PrintPathRole,
    layer_id: usize,
) -> SlicingPipeline {
    let layer_count = layer_id + 1;
    let layers = (0..layer_count)
        .map(|id| Layer::new(id, 0.2, 0.2 * (id + 1) as f64))
        .collect::<Vec<_>>();
    let layer_slices = layers
        .iter()
        .map(|layer| LayerSlice::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_contours = layers
        .iter()
        .map(|layer| LayerContours::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_perimeters = layers
        .iter()
        .map(|layer| LayerPerimeters::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_gap_fills = layers
        .iter()
        .map(|layer| LayerGapFills::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_infills = layers
        .iter()
        .map(|layer| LayerInfills::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_skirts = layers
        .iter()
        .map(|layer| LayerSkirts::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_brims = layers
        .iter()
        .map(|layer| LayerBrims::new(layer.id(), layer.print_z(), Vec::new()))
        .collect::<Vec<_>>();
    let layer_print_paths = crate::finalize_print_paths_with_layer_contours(
        layers
            .iter()
            .map(|layer| {
                let paths = if layer.id() == layer_id {
                    vec![
                        PrintPath::new(role, vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)])
                            .unwrap(),
                    ]
                } else {
                    Vec::new()
                };
                LayerPrintPaths::new(layer.id(), layer.print_z(), paths)
            })
            .collect::<Vec<_>>(),
        options,
        &layer_contours,
    )
    .unwrap();
    let print = build_print_domain(&layers, &layer_contours, &layer_print_paths).unwrap();
    let layer_toolpath_moves = generate_toolpath_moves(&layer_print_paths);
    let extrusion_options = options.extrusion_options().unwrap();
    let layer_extrusion_moves =
        generate_extrusion_moves(&layers, &layer_toolpath_moves, extrusion_options).unwrap();
    let layer_speed_moves =
        generate_speed_moves(&layer_extrusion_moves, options.speed_options().unwrap());
    let diagnostics = test_diagnostics(
        options,
        TestDiagnosticsInput {
            layers: &layers,
            layer_slices: &layer_slices,
            layer_contours: &layer_contours,
            layer_perimeters: &layer_perimeters,
            layer_infills: &layer_infills,
            layer_skirts: &layer_skirts,
            layer_brims: &layer_brims,
            layer_print_paths: &layer_print_paths,
            layer_toolpath_moves: &layer_toolpath_moves,
            layer_extrusion_moves: &layer_extrusion_moves,
            layer_speed_moves: &layer_speed_moves,
        },
    );

    SlicingPipeline {
        options: options.clone(),
        model: Model::new(InputFormat::Stl, Vec::new()),
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
        diagnostics,
    }
}

struct TestDiagnosticsInput<'a> {
    layers: &'a [Layer],
    layer_slices: &'a [LayerSlice],
    layer_contours: &'a [LayerContours],
    layer_perimeters: &'a [LayerPerimeters],
    layer_infills: &'a [LayerInfills],
    layer_skirts: &'a [LayerSkirts],
    layer_brims: &'a [LayerBrims],
    layer_print_paths: &'a [LayerPrintPaths],
    layer_toolpath_moves: &'a [LayerToolpathMoves],
    layer_extrusion_moves: &'a [LayerExtrusionMoves],
    layer_speed_moves: &'a [LayerSpeedMoves],
}

fn test_diagnostics(
    options: &SliceOptions,
    input: TestDiagnosticsInput<'_>,
) -> PipelineDiagnostics {
    PipelineDiagnostics {
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
        layer_count: input.layers.len(),
        total_segment_count: sum_by(input.layer_slices, |layer| layer.segments().len()),
        total_contour_count: sum_by(input.layer_contours, |layer| layer.contours().len()),
        total_perimeter_count: sum_by(input.layer_perimeters, |layer| layer.paths().len()),
        total_infill_count: sum_by(input.layer_infills, |layer| layer.paths().len()),
        total_skirt_path_count: sum_by(input.layer_skirts, |layer| layer.paths().len()),
        total_brim_path_count: sum_by(input.layer_brims, |layer| layer.paths().len()),
        total_print_path_count: sum_by(input.layer_print_paths, |layer| layer.paths().len()),
        total_toolpath_move_count: sum_by(input.layer_toolpath_moves, |layer| layer.moves().len()),
        total_extrusion_move_count: sum_by(input.layer_extrusion_moves, |layer| {
            layer.moves().len()
        }),
        total_speed_move_count: sum_by(input.layer_speed_moves, |layer| layer.moves().len()),
        total_extrusion_mm: input
            .layer_extrusion_moves
            .iter()
            .map(|layer| layer.total_extrusion_mm())
            .sum(),
        empty_layer_count: input
            .layer_contours
            .iter()
            .filter(|layer| layer.contours().is_empty())
            .count(),
        option_count: options.values().len(),
    }
}

fn sum_by<T>(items: &[T], count: impl Fn(&T) -> usize) -> usize {
    items.iter().map(count).sum()
}
