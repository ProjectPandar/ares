//! `SupportCommon.cpp::generate_raft_base`, including its brim trimming area.

use super::{
    contact_layer::TopContactLayer,
    first_layer::{FirstLayerInput, expand_support_first_layer},
    intermediate_heights::IntermediateLayer,
};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, difference_polygons_paths,
    offset_expolygons_paths, offset_paths, union_polygons_paths,
};
use crate::project_slice::parameters::SlicingParameters;
use crate::{ObjectOptions, ProcessBrimType};

#[derive(Clone, Copy)]
pub(super) enum RaftLayerType {
    Base,
    Interface,
}

pub(super) struct RaftLayer {
    pub(super) layer_type: RaftLayerType,
    pub(super) print_z: f64,
    pub(super) bottom_z: f64,
    pub(super) height: f64,
    pub(super) polygons: Vec<Polygon>,
    pub(super) contact_polygons: Option<Vec<Polygon>>,
}

pub(super) struct AdhesionInput<'a> {
    pub(super) object: &'a ObjectOptions,
    pub(super) parameters: &'a SlicingParameters,
    pub(super) first_object_slices: &'a [ExPolygon],
    // PrintObject::has_brim includes model-owned painted brim points.
    pub(super) has_brim: bool,
    pub(super) gap_xy_first_layer: f64,
    pub(super) first_layer_flow_width_scaled: i64,
    pub(super) scale: CoordinateScale,
}

pub(super) fn generate_raft_base(
    input: &AdhesionInput<'_>,
    tops: &mut [TopContactLayer],
    interfaces: &mut [IntermediateLayer],
    base_interfaces: &mut [IntermediateLayer],
    bases: &mut [IntermediateLayer],
) -> Result<Vec<RaftLayer>, ClipperError> {
    let parameters = input.parameters;
    let brim = brim_trimming(input)?;
    let raft_count = parameters.base_raft_layers + parameters.interface_raft_layers;
    if raft_count <= 1 {
        expand_support_first_layer(
            &FirstLayerInput {
                first_print_layer_height: parameters.first_print_layer_height,
                raft_contact_top_z: parameters.raft_contact_top_z,
                raft_interface_top_z: parameters.raft_interface_top_z,
                first_object_slices: input.first_object_slices,
                brim: &brim,
                first_layer_expansion: input.object.raft_first_layer_expansion.0,
                gap_xy_first_layer: input.gap_xy_first_layer,
                first_layer_flow_width_scaled: input.first_layer_flow_width_scaled,
                scale: input.scale,
            },
            tops,
            interfaces,
            base_interfaces,
            bases,
        )?;
        return Ok(Vec::new());
    }
    let fine = (0.5 / input.scale.factor()) as f32;
    let first_expansion =
        ((input.object.raft_first_layer_expansion.0 / input.scale.factor()) as f32 - fine).max(0.0);
    let contacts = tops.first().filter(|layer| {
        layer.print_z
            <= parameters
                .first_print_layer_height
                .max(parameters.raft_contact_top_z)
                + 1e-4
    });
    let mut interface_polygons = Vec::new();
    for polygons in contacts
        .into_iter()
        .map(|layer| layer.polygons.as_slice())
        .chain(
            [interfaces.first(), base_interfaces.first()]
                .into_iter()
                .flatten()
                .filter(|layer| {
                    layer.print_z - layer.height <= parameters.raft_interface_top_z + 1e-4
                })
                .map(|layer| layer.polygons.as_slice()),
        )
        .filter(|polygons| !polygons.is_empty())
    {
        interface_polygons.extend(offset_paths(polygons, fine, JoinType::Square, 0.0)?);
    }
    let mut base = Vec::new();
    let mut columns = Vec::new();
    let mut first_layer = Vec::new();
    if let Some(layer) = bases
        .first()
        .filter(|layer| layer.print_z - layer.height <= parameters.raft_interface_top_z + 1e-4)
    {
        if layer.print_z - layer.height > parameters.raft_interface_top_z - 1e-4 {
            base = layer.polygons.clone();
            columns = if interface_polygons.is_empty() {
                base.clone()
            } else {
                difference_polygons_paths(&base, &interface_polygons)?
            };
        } else {
            first_layer = layer.polygons.clone();
        }
    }
    if !interface_polygons.is_empty() {
        base.extend_from_slice(&interface_polygons);
        base = union_polygons_paths(&base)?;
    }
    first_layer.extend_from_slice(&base);
    first_layer = union_polygons_paths(&first_layer)?;
    let mut layers = vec![RaftLayer {
        layer_type: if parameters.base_raft_layers > 0 {
            RaftLayerType::Base
        } else {
            RaftLayerType::Interface
        },
        print_z: parameters.first_print_layer_height,
        bottom_z: 0.0,
        height: parameters.first_print_layer_height,
        polygons: if first_expansion > 0.0 {
            offset_paths(&first_layer, first_expansion, JoinType::Miter, 3.0)?
        } else {
            first_layer
        },
        contact_polygons: None,
    }];
    for (count, height, layer_type, polygons, contact_polygons) in [
        (
            parameters.base_raft_layers,
            parameters.base_raft_layer_height,
            RaftLayerType::Base,
            &base,
            None,
        ),
        (
            parameters.interface_raft_layers,
            parameters.interface_raft_layer_height,
            RaftLayerType::Interface,
            &interface_polygons,
            Some(&columns),
        ),
    ] {
        for _ in 1..count {
            let bottom_z = layers.last().unwrap().print_z;
            layers.push(RaftLayer {
                layer_type,
                print_z: bottom_z + height,
                bottom_z,
                height,
                polygons: polygons.clone(),
                contact_polygons: contact_polygons.cloned(),
            });
        }
    }
    Ok(layers)
}

fn brim_trimming(input: &AdhesionInput<'_>) -> Result<Vec<Polygon>, ClipperError> {
    if !input.has_brim {
        return Ok(Vec::new());
    }
    let outer = matches!(
        input.object.brim_type,
        ProcessBrimType::OuterOnly | ProcessBrimType::OuterAndInner
    );
    let inner = matches!(
        input.object.brim_type,
        ProcessBrimType::InnerOnly | ProcessBrimType::OuterAndInner
    );
    let gap = (input.object.brim_object_gap.0 / input.scale.factor()) as f32;
    let tolerance = f64::from((0.1 / input.scale.factor()) as f32);
    let mut brim = Vec::new();
    for expolygon in input.first_object_slices {
        if outer && inner {
            brim.extend(offset_expolygons_paths(
                std::slice::from_ref(expolygon),
                gap,
                JoinType::Miter,
                3.0,
            )?);
            continue;
        }
        if outer {
            brim.extend(offset_paths(
                std::slice::from_ref(expolygon.contour()),
                gap,
                JoinType::Round,
                tolerance,
            )?);
        } else {
            brim.push(expolygon.contour().clone());
        }
        if inner {
            let mut holes = expolygon.holes().to_vec();
            for hole in &mut holes {
                hole.reverse();
            }
            holes = offset_paths(&holes, -gap, JoinType::Round, tolerance)?;
            for hole in &mut holes {
                hole.reverse();
            }
            brim.extend(holes);
        } else {
            brim.extend_from_slice(expolygon.holes());
        }
    }
    union_polygons_paths(&brim)
}
