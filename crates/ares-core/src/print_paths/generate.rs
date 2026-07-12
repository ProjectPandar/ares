use super::shell_layers::solid_print_path_role;
use super::{LayerPrintPaths, PrintPath, PrintPathInput, PrintPathRole, ShellLayerOptions};
use crate::{
    Layer, LayerContours, SliceError, SliceOptions, bridge_support::fully_unsupported_layer,
};

pub fn generate_print_paths(
    input: PrintPathInput<'_>,
    shell_layers: ShellLayerOptions,
    is_infill_first: bool,
    bridge_no_support: bool,
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    generate_print_paths_with_bridge_policy(
        input,
        shell_layers,
        is_infill_first,
        crate::bridges::BridgeLayerPolicy::new(
            bridge_no_support,
            crate::bridges::ExtraBridgeLayer::Disabled,
            crate::bridges::CounterboreHoleBridging::None,
        ),
    )
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn finalize_print_paths(
    paths: Vec<LayerPrintPaths>,
    options: &SliceOptions,
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    finalize_print_paths_inner(paths, options, None)
}

pub(crate) fn finalize_print_paths_with_layer_contours(
    paths: Vec<LayerPrintPaths>,
    options: &SliceOptions,
    layer_contours: &[LayerContours],
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    finalize_print_paths_inner(paths, options, Some(layer_contours))
}

fn finalize_print_paths_inner(
    paths: Vec<LayerPrintPaths>,
    options: &SliceOptions,
    layer_contours: Option<&[LayerContours]>,
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    let paths = super::filter_short_gap_fill_paths(paths, options.filter_out_gap_fill_mm()?);
    let support_placement_options = layer_contours
        .map(|_| options.support_placement_options())
        .transpose()?;
    let support_z_distance = options.support_z_distance_options()?;
    let raft_options = options.raft_options()?;
    let raft_expansion = options.raft_expansion_mm()?;
    let raft_first_layer_expansion = options.raft_first_layer_expansion_mm()?;
    let raft_first_layer_density = options.raft_first_layer_density_percent()?;
    let enable_support = options.support_enable_options()?.enabled();
    let support_enabled = enable_support
        || support_z_distance.enforce_support_layers() > 0
        || raft_options.has_raft();
    let paths = super::apply_support_interface_top_layers(paths, options.values())?;
    let paths = super::apply_support_expansion(paths, options.values())?;
    let paths = super::apply_raft_expansion(paths, raft_options.layers(), raft_expansion);
    let paths = super::apply_raft_first_layer_expansion(
        paths,
        raft_options.has_raft(),
        raft_first_layer_expansion,
    );
    let explicit_snug_style = options
        .values()
        .get("support_style")
        .and_then(|value| value.as_str())
        == Some("snug");
    let support_type_for_snug = if explicit_snug_style {
        Some(options.support_type()?)
    } else {
        None
    };
    let snug_support_body = if let Some(support_type) = support_type_for_snug {
        options
            .support_style()?
            .resolve_for_support_type(support_type)
            .is_snug()
    } else {
        false
    };
    let paths = super::apply_support_style_snug(paths, snug_support_body);
    let extrusion_options = options.extrusion_options()?;
    let paths = if let (Some(layer_contours), Some(support_placement_options)) =
        (layer_contours, support_placement_options)
    {
        let support_type = support_type_for_snug.unwrap_or(options.support_type()?);
        let paths = super::apply_support_threshold_contacts(
            paths,
            layer_contours,
            enable_support && support_type.is_auto() && !support_type.is_tree(),
            options.support_threshold_options()?,
            extrusion_options.width_for_role(PrintPathRole::ExternalPerimeter),
        );
        let paths = super::apply_support_interface_top_layers(paths, options.values())?;
        let paths = super::apply_support_object_xy_distance(
            paths,
            layer_contours,
            support_placement_options.object_xy_distance_mm(),
            support_placement_options.object_first_layer_gap_mm(),
            raft_options.layers(),
        );
        let paths = super::apply_support_on_build_plate_only(
            paths,
            support_placement_options.on_build_plate_only(),
            raft_options.layers(),
        );
        let paths = super::apply_support_remove_small_overhang(
            paths,
            support_placement_options.remove_small_overhang(),
            extrusion_options.line_width_mm(),
        );
        let critical_regions_only = if support_placement_options.critical_regions_only() {
            support_type.is_tree() && support_type.is_auto()
        } else {
            false
        };
        let paths = super::apply_support_critical_regions_only(paths, critical_regions_only);
        let tree_support_options = options.tree_support_options()?;
        super::apply_tree_support_brim(
            paths,
            support_type.is_tree(),
            raft_options.layers(),
            tree_support_options.auto_brim(),
            tree_support_options.brim_width_mm(),
        )
    } else {
        paths
    };
    let support_angle = super::parse_support_angle(options.values())?;
    let support_interface_spacing = super::SupportInterfaceSpacingConfig::new(
        extrusion_options.width_for_role(PrintPathRole::SupportMaterialInterface),
        options.support_ironing()?,
        support_angle,
        support_z_distance,
    );
    let tree_support_wall_count = options.tree_support_options()?.wall_count();
    let paths = super::apply_support_base_pattern_spacing(
        paths,
        options.values(),
        super::SupportBaseSpacingConfig::new(
            extrusion_options.width_for_role(PrintPathRole::SupportMaterial),
            support_angle,
            raft_first_layer_density,
            tree_support_wall_count,
        ),
    )?;
    let paths =
        super::apply_support_interface_spacing(paths, options.values(), support_interface_spacing)?;
    let paths = super::apply_ironing(
        paths,
        crate::options::ironing_type::parse(options.values(), options.nozzle_diameters()?[0])?,
    );
    let paths = super::apply_support_ironing(
        paths,
        options.support_ironing()?,
        crate::options::ironing_flow::parse_support_ironing(options.values())?,
    );
    Ok(filter_disabled_support_paths(paths, support_enabled))
}

pub(crate) fn generate_print_paths_with_bridge_policy(
    input: PrintPathInput<'_>,
    shell_layers: ShellLayerOptions,
    is_infill_first: bool,
    bridge_policy: crate::bridges::BridgeLayerPolicy,
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    if input.skirts.len() != input.brims.len()
        || input.brims.len() != input.perimeters.len()
        || input.perimeters.len() != input.gap_fills.len()
        || input.gap_fills.len() != input.infills.len()
        || input
            .layer_contours
            .is_some_and(|contours| contours.len() != input.infills.len())
        || input
            .print_layers
            .is_some_and(|layers| layers.len() != input.infills.len())
    {
        return Err(SliceError::InvalidInput(
            "skirt, brim, perimeter, gap-fill and infill layer counts must match".to_owned(),
        ));
    }

    let layer_count = input.infills.len();

    input
        .skirts
        .iter()
        .zip(input.brims.iter())
        .zip(input.perimeters.iter())
        .zip(input.gap_fills.iter())
        .zip(input.infills.iter())
        .enumerate()
        .map(
            |(
                layer_index,
                ((((skirt_layer, brim_layer), perimeter_layer), gap_fill_layer), infill_layer),
            )| {
                let contour_layer = input
                    .layer_contours
                    .and_then(|layers| layers.get(layer_index));
                let print_layer = input
                    .print_layers
                    .and_then(|layers| layers.get(layer_index));
                let print_layer_mismatch = print_layer.is_some_and(|layer| {
                    layer.id() != skirt_layer.layer_id()
                        || layer.id() != brim_layer.layer_id()
                        || layer.id() != perimeter_layer.layer_id()
                        || layer.id() != gap_fill_layer.layer_id()
                        || layer.id() != infill_layer.layer_id()
                        || layer.print_z() != skirt_layer.print_z()
                        || layer.print_z() != brim_layer.print_z()
                        || layer.print_z() != perimeter_layer.print_z()
                        || layer.print_z() != gap_fill_layer.print_z()
                        || layer.print_z() != infill_layer.print_z()
                        || !layer.print_z().is_finite()
                        || !layer.height().is_finite()
                        || layer.height() <= 0.0
                });
                if skirt_layer.layer_id() != perimeter_layer.layer_id()
                    || skirt_layer.layer_id() != brim_layer.layer_id()
                    || skirt_layer.layer_id() != gap_fill_layer.layer_id()
                    || skirt_layer.layer_id() != infill_layer.layer_id()
                    || skirt_layer.print_z() != perimeter_layer.print_z()
                    || skirt_layer.print_z() != brim_layer.print_z()
                    || skirt_layer.print_z() != gap_fill_layer.print_z()
                    || skirt_layer.print_z() != infill_layer.print_z()
                    || brim_layer.layer_id() != perimeter_layer.layer_id()
                    || brim_layer.print_z() != perimeter_layer.print_z()
                    || perimeter_layer.layer_id() != gap_fill_layer.layer_id()
                    || perimeter_layer.print_z() != gap_fill_layer.print_z()
                    || gap_fill_layer.layer_id() != infill_layer.layer_id()
                    || gap_fill_layer.print_z() != infill_layer.print_z()
                    || perimeter_layer.layer_id() != infill_layer.layer_id()
                    || perimeter_layer.print_z() != infill_layer.print_z()
                    || contour_layer.is_some_and(|layer| {
                        layer.layer_id() != infill_layer.layer_id()
                            || layer.print_z() != infill_layer.print_z()
                    })
                    || print_layer_mismatch
                {
                    return Err(SliceError::InvalidInput(
                        "skirt, brim, perimeter, gap-fill, infill and print layers must match"
                            .to_owned(),
                    ));
                }

                let skirt_paths = skirt_layer
                    .paths()
                    .iter()
                    .map(|path| PrintPath::new(PrintPathRole::Skirt, path.points().to_vec()));
                let brim_paths = brim_layer
                    .paths()
                    .iter()
                    .map(|path| PrintPath::new(PrintPathRole::Brim, path.points().to_vec()));
                let perimeter_paths = perimeter_layer.paths().iter().map(|path| {
                    let role = match path.role() {
                        crate::PerimeterRole::External => PrintPathRole::ExternalPerimeter,
                        crate::PerimeterRole::Overhang => PrintPathRole::OverhangPerimeter,
                        crate::PerimeterRole::Internal => PrintPathRole::InternalPerimeter,
                    };
                    PrintPath::new(role, path.points().to_vec()).map(|print_path| {
                        print_path
                            .with_closed(path.is_closed())
                            .with_unsupported_span_mm(path.unsupported_span_mm())
                            .with_effective_line_width_mm(path.effective_line_width_mm())
                            .with_seam_gap_mm(path.seam_gap_mm())
                    })
                });
                let gap_fill_paths = gap_fill_layer
                    .paths()
                    .iter()
                    .map(|path| PrintPath::new(PrintPathRole::GapFill, path.points().to_vec()));
                let infill_paths = infill_layer.paths().iter().map(|path| {
                    let unsupported_bridge = bridge_policy.unsupported_bottom_bridge_enabled()
                        && input
                            .layer_contours
                            .is_some_and(|layers| fully_unsupported_layer(layers, layer_index));
                    let bridge_role = unsupported_bridge
                        || extra_external_bridge_layer(
                            input.layer_contours,
                            layer_index,
                            bridge_policy.bridge_no_support(),
                            bridge_policy.extra_bridge_layer(),
                        );
                    let role = match path.role() {
                        crate::InfillRole::Sparse => PrintPathRole::SparseInfill,
                        crate::InfillRole::InternalBridge => PrintPathRole::InternalBridge,
                        crate::InfillRole::BottomSurface => PrintPathRole::BottomSurface,
                        crate::InfillRole::TopSurface => PrintPathRole::TopSolidInfill,
                        crate::InfillRole::Solid => solid_infill_role(
                            shell_layers,
                            input.print_layers,
                            layer_index,
                            layer_count,
                            bridge_role,
                        ),
                    };
                    PrintPath::new(role, path.points().to_vec()).map(|print_path| {
                        print_path.with_effective_layer_height_mm(path.effective_layer_height_mm())
                    })
                });
                let paths = if is_infill_first && perimeter_layer.layer_id() != 0 {
                    skirt_paths
                        .chain(brim_paths)
                        .chain(infill_paths)
                        .chain(perimeter_paths)
                        .chain(gap_fill_paths)
                        .collect::<Result<Vec<_>, _>>()?
                } else {
                    skirt_paths
                        .chain(brim_paths)
                        .chain(perimeter_paths)
                        .chain(gap_fill_paths)
                        .chain(infill_paths)
                        .collect::<Result<Vec<_>, _>>()?
                };

                Ok(LayerPrintPaths::new(
                    perimeter_layer.layer_id(),
                    perimeter_layer.print_z(),
                    paths,
                ))
            },
        )
        .collect()
}

fn filter_disabled_support_paths(
    paths: Vec<LayerPrintPaths>,
    support_enabled: bool,
) -> Vec<LayerPrintPaths> {
    if support_enabled {
        return paths;
    }

    paths
        .into_iter()
        .map(|layer| {
            let paths = layer
                .paths()
                .iter()
                .filter(|path| !disabled_support_path(path))
                .cloned()
                .collect();
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect()
}

fn disabled_support_path(path: &PrintPath) -> bool {
    matches!(
        path.role(),
        PrintPathRole::SupportMaterial | PrintPathRole::SupportMaterialInterface
    ) || (path.role() == PrintPathRole::Ironing
        && path.extrusion_role() == Some(PrintPathRole::SupportMaterialInterface))
}

fn extra_external_bridge_layer(
    layer_contours: Option<&[LayerContours]>,
    layer_index: usize,
    bridge_no_support: bool,
    extra_bridge_layer: crate::bridges::ExtraBridgeLayer,
) -> bool {
    layer_index > 0
        && bridge_no_support
        && extra_bridge_layer.applies_to_external_bridge()
        && layer_contours.is_some_and(|layers| {
            fully_unsupported_layer(layers, layer_index - 1)
                && !fully_unsupported_layer(layers, layer_index)
        })
}

fn solid_infill_role(
    shell_layers: ShellLayerOptions,
    print_layers: Option<&[Layer]>,
    layer_index: usize,
    layer_count: usize,
    unsupported_bridge: bool,
) -> PrintPathRole {
    match print_layers {
        Some(print_layers) => {
            shell_layers.solid_role(print_layers, layer_index, unsupported_bridge)
        }
        None => solid_print_path_role(layer_index, layer_count, shell_layers, unsupported_bridge),
    }
}
