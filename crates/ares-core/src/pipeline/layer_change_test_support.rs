use super::*;
use crate::{
    ExtrusionMove, InputFormat, Layer, LayerBrims, LayerContours, LayerExtrusionMoves,
    LayerGapFills, LayerInfills, LayerPerimeters, LayerPrintPaths, LayerSkirts, LayerSlice,
    LayerToolpathMoves, Model, Point2, PrintPath, PrintPathRole, SliceOptions, ToolpathMove,
    ToolpathMoveKind, build_print_domain, generate_extrusion_moves, generate_speed_moves,
    generate_toolpath_moves,
};

pub(crate) fn role_layers_pipeline(
    options: &SliceOptions,
    roles_by_layer: Vec<Vec<PrintPathRole>>,
) -> SlicingPipeline {
    let layers = (0..roles_by_layer.len())
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
    let layer_print_paths = role_layer_print_paths(&layers, roles_by_layer);
    let print = build_print_domain(&layers, &layer_contours, &layer_print_paths).unwrap();
    let layer_toolpath_moves = generate_toolpath_moves(&layer_print_paths);
    let extrusion_options = options.extrusion_options().unwrap();
    let layer_extrusion_moves =
        generate_extrusion_moves(&layers, &layer_toolpath_moves, extrusion_options).unwrap();
    let layer_speed_moves =
        generate_speed_moves(&layer_extrusion_moves, options.speed_options().unwrap());
    let diagnostics = diagnostics(
        options,
        DiagnosticInput {
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

pub(crate) fn pending_travel_layer_boundary_pipeline(options: &SliceOptions) -> SlicingPipeline {
    let mut pipeline = role_layers_pipeline(
        options,
        vec![
            vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
            vec![PrintPathRole::SparseInfill],
        ],
    );
    let layer_id = pipeline.layers[0].id();
    let print_z = pipeline.layers[0].print_z();
    let first_move = pipeline.layer_extrusion_moves[0].moves()[0];
    let first_print = pipeline.layer_extrusion_moves[0].moves()[1];
    let total_extrusion_mm = pipeline.layer_extrusion_moves[0].total_extrusion_mm();
    let trailing_travel = ExtrusionMove::new(
        ToolpathMoveKind::Travel,
        PrintPathRole::SparseInfill,
        Point2::new(2.0, 0.0),
        None,
    );
    pipeline.layer_extrusion_moves[0] = LayerExtrusionMoves::new(
        layer_id,
        print_z,
        vec![first_move, first_print, trailing_travel],
        total_extrusion_mm,
    );
    pipeline.layer_speed_moves = generate_speed_moves(
        &pipeline.layer_extrusion_moves,
        options.speed_options().unwrap(),
    );
    pipeline
}

pub(crate) fn zero_distance_travel_after_print_pipeline(options: &SliceOptions) -> SlicingPipeline {
    let mut pipeline = role_layers_pipeline(
        options,
        vec![vec![
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::ExternalPerimeter,
        ]],
    );
    let layer_id = pipeline.layers[0].id();
    let print_z = pipeline.layers[0].print_z();
    let first_move = pipeline.layer_extrusion_moves[0].moves()[0];
    let first_print = pipeline.layer_extrusion_moves[0].moves()[1];
    let total_extrusion_mm = pipeline.layer_extrusion_moves[0].total_extrusion_mm();
    let zero_travel = ExtrusionMove::new(
        ToolpathMoveKind::Travel,
        PrintPathRole::ExternalPerimeter,
        first_print.point(),
        None,
    );
    pipeline.layer_extrusion_moves[0] = LayerExtrusionMoves::new(
        layer_id,
        print_z,
        vec![first_move, first_print, zero_travel],
        total_extrusion_mm,
    );
    pipeline.layer_toolpath_moves[0] = LayerToolpathMoves::new(
        layer_id,
        print_z,
        vec![
            ToolpathMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::ExternalPerimeter,
                first_move.point(),
            ),
            ToolpathMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                first_print.point(),
            ),
            ToolpathMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::ExternalPerimeter,
                first_print.point(),
            ),
        ],
    );
    pipeline.layer_speed_moves = generate_speed_moves(
        &pipeline.layer_extrusion_moves,
        options.speed_options().unwrap(),
    );
    pipeline
}

pub(crate) fn print_first_after_layer_change_pipeline(options: &SliceOptions) -> SlicingPipeline {
    let mut pipeline = role_layers_pipeline(
        options,
        vec![
            vec![PrintPathRole::ExternalPerimeter],
            vec![PrintPathRole::ExternalPerimeter],
        ],
    );
    let layer_id = pipeline.layers[1].id();
    let print_z = pipeline.layers[1].print_z();
    let first_print = pipeline.layer_extrusion_moves[1].moves()[1];
    let total_extrusion_mm = pipeline.layer_extrusion_moves[1].total_extrusion_mm();
    pipeline.layer_extrusion_moves[1] =
        LayerExtrusionMoves::new(layer_id, print_z, vec![first_print], total_extrusion_mm);
    pipeline.layer_toolpath_moves[1] = LayerToolpathMoves::new(
        layer_id,
        print_z,
        vec![ToolpathMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            first_print.point(),
        )],
    );
    pipeline.layer_speed_moves = generate_speed_moves(
        &pipeline.layer_extrusion_moves,
        options.speed_options().unwrap(),
    );
    pipeline
}

fn role_layer_print_paths(
    layers: &[Layer],
    roles_by_layer: Vec<Vec<PrintPathRole>>,
) -> Vec<LayerPrintPaths> {
    layers
        .iter()
        .zip(roles_by_layer)
        .map(|(layer, roles)| {
            let paths = roles
                .into_iter()
                .enumerate()
                .map(|(index, role)| {
                    let x = index as f64 * 2.0;
                    PrintPath::new(role, vec![Point2::new(x, 0.0), Point2::new(x + 1.0, 0.0)])
                        .unwrap()
                })
                .collect::<Vec<_>>();
            LayerPrintPaths::new(layer.id(), layer.print_z(), paths)
        })
        .collect()
}

struct DiagnosticInput<'a> {
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

fn diagnostics(options: &SliceOptions, input: DiagnosticInput<'_>) -> PipelineDiagnostics {
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
        total_segment_count: input
            .layer_slices
            .iter()
            .map(|layer| layer.segments().len())
            .sum(),
        total_contour_count: input
            .layer_contours
            .iter()
            .map(|layer| layer.contours().len())
            .sum(),
        total_perimeter_count: input
            .layer_perimeters
            .iter()
            .map(|layer| layer.paths().len())
            .sum(),
        total_infill_count: input
            .layer_infills
            .iter()
            .map(|layer| layer.paths().len())
            .sum(),
        total_skirt_path_count: input
            .layer_skirts
            .iter()
            .map(|layer| layer.paths().len())
            .sum(),
        total_brim_path_count: input
            .layer_brims
            .iter()
            .map(|layer| layer.paths().len())
            .sum(),
        total_print_path_count: input
            .layer_print_paths
            .iter()
            .map(|layer| layer.paths().len())
            .sum(),
        total_toolpath_move_count: input
            .layer_toolpath_moves
            .iter()
            .map(|layer| layer.moves().len())
            .sum(),
        total_extrusion_move_count: input
            .layer_extrusion_moves
            .iter()
            .map(|layer| layer.moves().len())
            .sum(),
        total_speed_move_count: input
            .layer_speed_moves
            .iter()
            .map(|layer| layer.moves().len())
            .sum(),
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
