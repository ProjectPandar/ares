use crate::{
    Layer, LayerBrims, LayerContours, LayerExtrusionMoves, LayerGapFills, LayerInfills,
    LayerPerimeters, LayerPrintPaths, LayerSkirts, LayerSlice, LayerSpeedMoves, LayerToolpathMoves,
    Model, Print, PrintPathRole, SliceError, SliceOptions, SolidSurfaceGapFillInput,
    ToolpathMoveKind, append_solid_surface_gap_fills, build_print_domain, generate_brims,
    generate_extrusion_moves, generate_gap_fills, generate_perimeters, generate_speed_moves,
    generate_toolpath_moves,
    infills::{InfillBridgeContext, generate_infills_with_bridge_context},
    load_model, plan_layers,
    skirts::generate_skirts_after_brims,
    slice_layers, stitch_printable,
};

mod diagnostics;

pub use diagnostics::{PipelineDiagnostics, PipelineStage};

#[derive(Clone, Debug, PartialEq)]
pub struct SlicingPipeline {
    options: SliceOptions,
    model: Model,
    layers: Vec<Layer>,
    layer_slices: Vec<LayerSlice>,
    layer_contours: Vec<LayerContours>,
    layer_perimeters: Vec<LayerPerimeters>,
    layer_gap_fills: Vec<LayerGapFills>,
    layer_infills: Vec<LayerInfills>,
    layer_skirts: Vec<LayerSkirts>,
    layer_brims: Vec<LayerBrims>,
    layer_print_paths: Vec<LayerPrintPaths>,
    print: Print,
    layer_toolpath_moves: Vec<LayerToolpathMoves>,
    layer_extrusion_moves: Vec<LayerExtrusionMoves>,
    layer_speed_moves: Vec<LayerSpeedMoves>,
    diagnostics: PipelineDiagnostics,
}

impl SlicingPipeline {
    pub const fn options(&self) -> &SliceOptions {
        &self.options
    }

    pub const fn model(&self) -> &Model {
        &self.model
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn layer_slices(&self) -> &[LayerSlice] {
        &self.layer_slices
    }

    pub fn layer_contours(&self) -> &[LayerContours] {
        &self.layer_contours
    }

    pub fn layer_perimeters(&self) -> &[LayerPerimeters] {
        &self.layer_perimeters
    }

    pub fn layer_gap_fills(&self) -> &[LayerGapFills] {
        &self.layer_gap_fills
    }

    pub fn layer_infills(&self) -> &[LayerInfills] {
        &self.layer_infills
    }

    pub fn layer_skirts(&self) -> &[LayerSkirts] {
        &self.layer_skirts
    }

    pub fn layer_brims(&self) -> &[LayerBrims] {
        &self.layer_brims
    }

    pub fn layer_print_paths(&self) -> &[LayerPrintPaths] {
        &self.layer_print_paths
    }

    pub const fn print(&self) -> &Print {
        &self.print
    }

    pub fn layer_toolpath_moves(&self) -> &[LayerToolpathMoves] {
        &self.layer_toolpath_moves
    }

    pub fn layer_extrusion_moves(&self) -> &[LayerExtrusionMoves] {
        &self.layer_extrusion_moves
    }

    pub fn layer_speed_moves(&self) -> &[LayerSpeedMoves] {
        &self.layer_speed_moves
    }

    pub const fn diagnostics(&self) -> &PipelineDiagnostics {
        &self.diagnostics
    }
}

pub fn run_slicing_pipeline(
    input: impl AsRef<[u8]>,
    options: &SliceOptions,
) -> Result<SlicingPipeline, SliceError> {
    let mut options = options.clone();
    options.normalize_fdm(0)?;
    options.validate_slicing_print_sequence()?;
    options.validate_slicing_different_extruders()?;
    options.validate_slicing_physical_extruder_map()?;
    options.validate_nozzle_temperature_ranges()?;
    options.support_enable_options()?.consume_runtime();
    options.support_type()?;
    options.support_style()?;
    options.support_placement_options()?.consume_runtime();
    options.support_threshold_options()?.consume_runtime();
    options
        .support_interface_not_for_body_options()?
        .consume_runtime();
    options.tree_support_options()?.consume_runtime();
    let model = crate::model_shrinkage::apply(load_model(input)?, &options)?;
    options.validate_model_bed_excluded_area(&model)?;
    let layers = plan_layers(&model, &options)?;
    crate::printable_height::validate_layers(&layers, &options)?;
    let layer_slices = slice_layers(&model, &layers)?;
    let layer_contours = stitch_printable(&layer_slices, &options)?;
    let layer_perimeters = generate_perimeters(&layer_contours, options.perimeter_options()?)?;
    let mut layer_gap_fills = generate_gap_fills(
        &layer_contours,
        options.perimeter_options()?,
        options
            .speed_options()?
            .speed_for_role(ToolpathMoveKind::Print, PrintPathRole::GapFill),
    )?;
    let bridge_options = options.bridge_options()?;
    let infill_options = options.infill_options()?;
    append_solid_surface_gap_fills(
        &mut layer_gap_fills,
        SolidSurfaceGapFillInput {
            print_layers: &layers,
            layer_contours: &layer_contours,
            infill_options: &infill_options,
            target: options.gap_fill_target()?,
            bridge_no_support: bridge_options.bridge_no_support(),
            extra_bridge_layer: bridge_options.extra_bridge_layer(),
            counterbore_hole_bridging: bridge_options.counterbore_hole_bridging(),
        },
    )?;
    let layer_infills = generate_infills_with_bridge_context(
        &layers,
        &layer_contours,
        infill_options,
        Some(InfillBridgeContext::new(
            &layer_contours,
            bridge_options.bridge_no_support(),
            bridge_options.extra_bridge_layer(),
            bridge_options.counterbore_hole_bridging(),
        )),
    )?;
    let extrusion_options = options.extrusion_options()?;
    let layer_brims = generate_brims(
        &layer_contours,
        options.brim_options()?,
        extrusion_options.width_for_role(PrintPathRole::Brim),
    )?;
    let skirt_extrusion_per_mm = extrusion_options
        .extrusion_per_mm(PrintPathRole::Skirt, options.initial_layer_print_height()?)?;
    let layer_skirts = generate_skirts_after_brims(
        &layer_contours,
        &layer_brims,
        options.skirt_options()?,
        extrusion_options.width_for_role(PrintPathRole::Skirt),
        skirt_extrusion_per_mm,
    )?;
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
            options.shell_layer_options()?,
            options.is_infill_first()?,
            crate::bridges::BridgeLayerPolicy::new(
                bridge_options.bridge_no_support(),
                bridge_options.extra_bridge_layer(),
                bridge_options.counterbore_hole_bridging(),
            ),
        )?,
        &options,
        &layer_contours,
    )?;
    let print = build_print_domain(&layers, &layer_contours, &layer_print_paths)?;
    let layer_toolpath_moves = generate_toolpath_moves(&layer_print_paths);
    let layer_extrusion_moves =
        generate_extrusion_moves(&layers, &layer_toolpath_moves, extrusion_options)?;
    let layer_speed_moves = generate_speed_moves(&layer_extrusion_moves, options.speed_options()?);
    let total_brim_path_count = layer_brims.iter().map(|layer| layer.paths().len()).sum();
    let diagnostics = PipelineDiagnostics {
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
        input_format: model.format(),
        triangle_count: model.triangles().len(),
        layer_count: layers.len(),
        total_segment_count: layer_slices
            .iter()
            .map(|layer| layer.segments().len())
            .sum(),
        total_contour_count: layer_contours
            .iter()
            .map(|layer| layer.contours().len())
            .sum(),
        total_perimeter_count: layer_perimeters
            .iter()
            .map(|layer| layer.paths().len())
            .sum(),
        total_infill_count: layer_infills.iter().map(|layer| layer.paths().len()).sum(),
        total_skirt_path_count: layer_skirts.iter().map(|layer| layer.paths().len()).sum(),
        total_brim_path_count,
        total_print_path_count: layer_print_paths
            .iter()
            .map(|layer| layer.paths().len())
            .sum(),
        total_toolpath_move_count: layer_toolpath_moves
            .iter()
            .map(|layer| layer.moves().len())
            .sum(),
        total_extrusion_move_count: layer_extrusion_moves
            .iter()
            .map(|layer| layer.moves().len())
            .sum(),
        total_speed_move_count: layer_speed_moves
            .iter()
            .map(|layer| layer.moves().len())
            .sum(),
        total_extrusion_mm: layer_extrusion_moves
            .iter()
            .map(|layer| layer.total_extrusion_mm())
            .sum(),
        empty_layer_count: layer_contours
            .iter()
            .filter(|layer| layer.contours().is_empty())
            .count(),
        option_count: options.values().len(),
    };

    Ok(SlicingPipeline {
        options,
        model,
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
    })
}
#[cfg(test)]
pub(crate) mod layer_change_test_support;
#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
