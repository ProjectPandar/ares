//! Normal Grid/Snug `SupportCommon.cpp::generate_interface_layers`.

mod projection;

use super::{
    bottom_contacts::BottomContactLayer,
    contact_layer::TopContactLayer,
    interface_counts::InterfaceCounts,
    intermediate_heights::{IntermediateLayer, SupportLayerType},
    toolpath_parameters::ToolpathParameters,
};
use crate::ProcessSupportStyle;
use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, Polygon, difference_polygons_paths,
    intersection_polygons_paths, mutable_polygon::smooth_outward, offset_paths,
    union_safety_offset_polygons,
};

pub(super) struct GridInterfacesInput<'a> {
    pub(super) style: ProcessSupportStyle,
    pub(super) parameters: &'a ToolpathParameters,
    pub(super) scale: CoordinateScale,
    pub(super) counts: &'a InterfaceCounts,
    pub(super) tops: &'a [TopContactLayer],
    pub(super) bottoms: &'a [BottomContactLayer],
}

pub(super) struct InterfaceLayers {
    pub(super) interfaces: Vec<IntermediateLayer>,
    pub(super) base_interfaces: Vec<IntermediateLayer>,
}

pub(super) fn generate_grid_interface_layers(
    input: &GridInterfacesInput<'_>,
    bases: &mut [IntermediateLayer],
) -> Result<InterfaceLayers, ClipperError> {
    let mut output = InterfaceLayers {
        interfaces: Vec::new(),
        base_interfaces: Vec::new(),
    };
    if bases.is_empty() || input.counts.top + input.counts.bottom == 0 {
        return Ok(output);
    }
    // SupportCommon.cpp:220–233; SupportParameters.hpp:49–50 defines contacts as n > 0.
    let counts = input
        .counts
        .projected(input.style == ProcessSupportStyle::Organic);
    let input = &GridInterfacesInput {
        counts: &counts,
        ..*input
    };
    let mut first_top = -1;
    let mut first_bottom = -1;
    for index in 0..bases.len() {
        let (top, top_base) = projection::top(input, bases, index, &mut first_top);
        let (bottom, bottom_base) = projection::bottom(input, bases, index, &mut first_bottom);
        let kind = if top.is_empty() {
            SupportLayerType::BottomInterface
        } else {
            SupportLayerType::TopInterface
        };
        let interface = if bottom.is_empty() && top.is_empty() {
            None
        } else {
            insert_layer(input, &mut bases[index], (bottom, top), None, kind)?
        };
        let base_interface = if bottom_base.is_empty() && top_base.is_empty() {
            None
        } else {
            insert_layer(
                input,
                &mut bases[index],
                (bottom_base, top_base),
                interface.as_ref().map(|layer| layer.polygons.as_slice()),
                SupportLayerType::Base,
            )?
        };
        if let Some(layer) = interface {
            output.interfaces.push(layer);
        }
        if let Some(layer) = base_interface
            && !layer.polygons.is_empty()
        {
            output.base_interfaces.push(layer);
        }
    }
    Ok(output)
}

fn insert_layer(
    input: &GridInterfacesInput<'_>,
    base: &mut IntermediateLayer,
    (bottom, top): (Vec<Polygon>, Vec<Polygon>),
    subtract: Option<&[Polygon]>,
    layer_type: SupportLayerType,
) -> Result<Option<IntermediateLayer>, ClipperError> {
    let mut polygons = regularize(input, bottom, input.parameters.bottom_interface_density)?;
    polygons.extend(regularize(
        input,
        top,
        input.parameters.top_interface_density,
    )?);
    let mut polygons = intersection_polygons_paths(&polygons, &base.polygons)?;
    if polygons.is_empty() {
        return Ok(None);
    }
    base.polygons = difference_polygons_paths(&base.polygons, &polygons)?;
    if let Some(subtract) = subtract {
        polygons = difference_polygons_paths(&polygons, subtract)?;
    }
    Ok(Some(IntermediateLayer {
        layer_type,
        print_z: base.print_z,
        bottom_z: base.bottom_z,
        height: base.height,
        polygons,
    }))
}

fn regularize(
    input: &GridInterfacesInput<'_>,
    polygons: Vec<Polygon>,
    density: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    if polygons.is_empty() {
        Ok(polygons)
    } else if input.style != ProcessSupportStyle::Grid {
        let spacing = input
            .scale
            .scaled_delta(f64::from(input.parameters.interface_flow.spacing))
            as i64;
        let smoothing = spacing as f64 * 1.5;
        let expanded = offset_paths(
            &polygons,
            (smoothing + spacing as f64 / density) as f32,
            JoinType::Square,
            0.0,
        )?;
        let closed = offset_paths(&expanded, -(smoothing as f32), JoinType::Square, 0.0)?;
        Ok(smooth_outward(closed, smoothing as i64, input.scale))
    } else {
        union_safety_offset_polygons(&polygons)
    }
}
