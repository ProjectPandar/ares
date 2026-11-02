//! Geometry prefix of `PrintObjectSupportMaterial::generate`, through layer installation.
//! Support toolpaths and public project integration follow it.

use super::{
    adhesion::{AdhesionInput, RaftLayer, generate_raft_base},
    base_layers::{BaseLayersInput, generate_base_layers},
    bottom_contacts::BottomContactLayer,
    bottom_projection::{BottomProjectionInput, bottom_contact_layers_and_layer_support_areas},
    contact_layer::TopContactLayer,
    contact_placement::ContactPlacement,
    detect_contacts::buildplate_covered,
    install_layers::{InstalledSupportLayer, generate_support_layers},
    interface_counts::InterfaceCounts,
    interface_grid::{GridInterfacesInput, generate_grid_interface_layers},
    intermediate_heights::{IntermediateLayer, raft_and_intermediate_support_layers},
    layer_input::{LayerAnnotations, LayerInput},
    overhangs::SharpTails,
    toolpath_parameters::ToolpathParameters,
    top_contacts::{TopContactsInput, top_contact_layers},
    trim_object::{
        ObjectTrimming, SupportLayerView, TrimLayer, TrimRegion, trim_support_layers_by_object,
    },
    trim_top_contacts::trim_top_contacts_by_bottom_contacts,
};
use crate::geometry::support_grid::SupportGridParams;
use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, Polygon, offset_paths, union_polygons_paths,
};

pub(super) struct GridSupportInput<'a> {
    pub(super) toolpath_parameters: &'a ToolpathParameters,
    pub(super) placement: ContactPlacement<'a>,
    pub(super) layers: &'a [LayerInput<'a>],
    // Slice-support volumes plus projected custom facets, before blocker expansion.
    pub(super) enforcers: &'a [Vec<Polygon>],
    pub(super) blockers: &'a [Vec<Polygon>],
    pub(super) top_surfaces: &'a [Vec<Polygon>],
    pub(super) bridging_heights: &'a [Option<f64>],
    pub(super) trim_regions: &'a [Vec<TrimRegion<'a>>],
    pub(super) interface_counts: &'a InterfaceCounts,
    // Configured line_width.get_abs_value(interface_nozzle), not resolved Flow width.
    pub(super) extrusion_width_scaled: f64,
    pub(super) bottom_interface_height: f64,
    pub(super) support_width_scaled: f32,
    pub(super) gap_xy: f64,
    pub(super) gap_xy_first_layer: f64,
    pub(super) first_layer_flow_width_scaled: i64,
    pub(super) has_brim: bool,
    pub(super) grid: SupportGridParams,
    pub(super) scale: CoordinateScale,
}

pub(super) struct GeneratedSupportGeometry {
    pub(super) ownership: LayerOwnership,
    pub(super) support_layers: Vec<InstalledSupportLayer>,
    pub(super) rafts: Vec<RaftLayer>,
    pub(super) tops: Vec<TopContactLayer>,
    pub(super) bottoms: Vec<BottomContactLayer>,
    pub(super) bases: Vec<IntermediateLayer>,
    pub(super) interfaces: Vec<IntermediateLayer>,
    pub(super) base_interfaces: Vec<IntermediateLayer>,
    pub(super) sharp_tails: Vec<SharpTails>,
}

pub(super) enum LayerOwnership {
    Distinct,
    Organic {
        interfaces: Vec<usize>,
        base_interfaces: Vec<usize>,
    },
}

impl LayerOwnership {
    pub(super) fn interface(&self, index: usize, base: bool) -> usize {
        match self {
            Self::Distinct => index,
            Self::Organic {
                interfaces,
                base_interfaces,
            } => {
                if base {
                    base_interfaces[index]
                } else {
                    interfaces[index]
                }
            }
        }
    }
}

pub(super) fn generate_grid_support_geometry(
    input: &GridSupportInput<'_>,
) -> Result<GeneratedSupportGeometry, ClipperError> {
    let object = input.placement.object;
    let has_support = object.enable_support.0 || object.enforce_support_layers.0 != 0;
    let mut covered = if has_support && object.support_on_build_plate_only.0 {
        Some(buildplate_covered(input.layers, input.scale)?)
    } else {
        None
    };
    let blockers = expand_blockers(input.blockers, input.scale)?;
    let annotations = (0..input.layers.len())
        .map(|index| LayerAnnotations {
            buildplate_covered: covered.as_ref().map(|layers| layers[index].as_slice()),
            enforcers: annotation_at(input.enforcers, index),
            blockers: annotation_at(&blockers, index),
        })
        .collect::<Vec<_>>();
    let mut sharp_tails = (0..input.layers.len())
        .map(|_| SharpTails::default())
        .collect::<Vec<_>>();
    let mut tops = top_contact_layers(
        &TopContactsInput {
            placement: input.placement,
            layers: input.layers,
            annotations: &annotations,
            bridging_heights: input.bridging_heights,
            grid: input.grid,
            scale: input.scale,
        },
        input.extrusion_width_scaled,
        &mut sharp_tails,
    )?;
    let mut output = GeneratedSupportGeometry {
        ownership: LayerOwnership::Distinct,
        support_layers: Vec::new(),
        rafts: Vec::new(),
        tops: Vec::new(),
        bottoms: Vec::new(),
        bases: Vec::new(),
        interfaces: Vec::new(),
        base_interfaces: Vec::new(),
        sharp_tails,
    };
    if tops.is_empty() {
        return Ok(output);
    }
    let trim_layers = input
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| TrimLayer {
            slices: layer.slices,
            sharp_tails: &output.sharp_tails[index].polygons,
            regions: &input.trim_regions[index],
        })
        .collect::<Vec<_>>();
    let parameters = input.placement.parameters;
    let trimming = ObjectTrimming {
        planned: input.placement.layers,
        layers: &trim_layers,
        raft_contact_top_z: parameters.raft_contact_top_z,
        gap_extra_above: parameters.gap_support_object,
        gap_extra_below: parameters.gap_object_support,
        gap_xy: input.gap_xy,
        zero_gap_interface_top: parameters.zero_gap_interface_top,
        thick_bridges: input.placement.object.thick_bridges.0,
        scale: input.scale,
    };
    let bottom = bottom_contact_layers_and_layer_support_areas(
        &BottomProjectionInput {
            placement: &input.placement,
            layers: input.layers,
            top_surfaces: input.top_surfaces,
            bottom_interface_height: input.bottom_interface_height,
            support_width_scaled: input.support_width_scaled,
            grid: input.grid,
            scale: input.scale,
        },
        &mut tops,
        covered.as_deref_mut(),
        &trimming,
    )?;
    let mut bases =
        raft_and_intermediate_support_layers(&input.placement, &bottom.contacts, &mut tops);
    trim_support_layers_by_object(
        &trimming,
        tops.iter_mut().map(|layer| SupportLayerView {
            print_z: layer.print_z,
            bottom_z: layer.bottom_z,
            height: layer.height,
            polygons: &mut layer.polygons,
        }),
    )?;
    generate_base_layers(
        &BaseLayersInput {
            planned: input.placement.layers,
            bottoms: &bottom.contacts,
            tops: &tops,
            support_areas: &bottom.support_areas,
            trimming: &trimming,
        },
        &mut bases,
    )?;
    trim_top_contacts_by_bottom_contacts(&bottom.contacts, &mut tops)?;
    let interfaces = generate_grid_interface_layers(
        &GridInterfacesInput {
            style: super::style::resolve(object.support_type, object.support_style),
            parameters: input.toolpath_parameters,
            scale: input.scale,
            counts: input.interface_counts,
            tops: &tops,
            bottoms: &bottom.contacts,
        },
        &mut bases,
    )?;
    output.tops = tops;
    output.bottoms = bottom.contacts;
    output.bases = bases;
    output.interfaces = interfaces.interfaces;
    output.base_interfaces = interfaces.base_interfaces;
    output.rafts = generate_raft_base(
        &AdhesionInput {
            object,
            parameters,
            first_object_slices: input.layers[0].slices,
            has_brim: input.has_brim,
            gap_xy_first_layer: input.gap_xy_first_layer,
            first_layer_flow_width_scaled: input.first_layer_flow_width_scaled,
            scale: input.scale,
        },
        &mut output.tops,
        &mut output.interfaces,
        &mut output.base_interfaces,
        &mut output.bases,
    )?;
    output.support_layers = generate_support_layers(&mut output);
    Ok(output)
}

fn annotation_at(layers: &[Vec<Polygon>], index: usize) -> &[Polygon] {
    // Source annotations use an empty vector to mean no enforcer/blocker volumes.
    if layers.is_empty() {
        &[]
    } else {
        &layers[index]
    }
}

fn expand_blockers(
    layers: &[Vec<Polygon>],
    scale: CoordinateScale,
) -> Result<Vec<Vec<Polygon>>, ClipperError> {
    let expansion = (1000.0 * (1e-4 / scale.factor())) as f32;
    layers
        .iter()
        .map(|polygons| {
            if polygons.is_empty() {
                Ok(Vec::new())
            } else {
                offset_paths(
                    &union_polygons_paths(polygons)?,
                    expansion,
                    JoinType::Miter,
                    3.0,
                )
            }
        })
        .collect()
}
