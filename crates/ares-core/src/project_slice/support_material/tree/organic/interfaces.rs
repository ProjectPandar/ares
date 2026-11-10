//! Organic branch of `SupportCommon.cpp::generate_interface_layers:47-239`.

use super::interface_placer::{GeneratorLayer, LayerType, RoofLayers};
use crate::{
    geometry::{
        ClipperError, CoordinateScale, JoinType, Polygon, difference_polygons_paths,
        intersection_polygons_paths, mutable_polygon::smooth_outward, offset_paths,
        union_polygons_paths_pair,
    },
    project_slice::support_material::{
        interface_counts::InterfaceCounts, toolpath_parameters::ToolpathParameters,
    },
};

const EPSILON: f64 = 1e-4;

pub(super) fn generate(
    roofs: &mut RoofLayers,
    counts: &InterfaceCounts,
    parameters: &ToolpathParameters,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let bases = indices(&roofs.intermediate_layers);
    let tops = indices(&roofs.top_contacts);
    let bottoms = indices(&roofs.bottom_contacts);
    let previous_interfaces = indices(&roofs.top_interfaces);
    let previous_base_interfaces = indices(&roofs.top_base_interfaces);
    let mut interfaces = Vec::new();
    let mut base_interfaces = Vec::new();
    if !bases.is_empty() && counts.top + counts.bottom > 0 {
        let counts = &counts.projected(true);
        for (index, &base) in bases.iter().enumerate() {
            let (top, top_base) = project_top(roofs, counts, &bases, &tops, index);
            let (bottom, bottom_base) = project_bottom(roofs, counts, &bases, &bottoms, index);
            let z = roofs.storage[base].print_z;
            let existing = same_layer(roofs, &previous_interfaces, z);
            let kind = if top.is_empty() {
                LayerType::BottomInterface
            } else {
                LayerType::TopInterface
            };
            let interface = insert(
                roofs,
                base,
                (bottom, top, existing),
                (None, kind),
                (parameters, scale),
            )?;
            interfaces.extend(interface);
            let existing = same_layer(roofs, &previous_base_interfaces, z);
            let base_interface = insert(
                roofs,
                base,
                (bottom_base, top_base, existing),
                (interface, LayerType::Base),
                (parameters, scale),
            )?;
            base_interfaces.extend(base_interface);
        }
        roofs.top_interfaces = merge(roofs, interfaces, previous_interfaces);
        roofs.top_base_interfaces = merge(roofs, base_interfaces, previous_base_interfaces);
    } else {
        // The source returns empty output, including precomputed roofs, in this branch.
        roofs.top_interfaces.clear();
        roofs.top_base_interfaces.clear();
    }
    Ok(())
}

fn project_top(
    roofs: &RoofLayers,
    counts: &InterfaceCounts,
    bases: &[usize],
    contacts: &[usize],
    index: usize,
) -> (Vec<Polygon>, Vec<Polygon>) {
    let mut interface = Vec::new();
    let mut base = Vec::new();
    if counts.top == 0 {
        return (interface, base);
    }
    let current = &roofs.storage[bases[index]];
    let top_z = roofs.storage[bases[(index + counts.top - 1).min(bases.len() - 1)]].print_z;
    let interface_z = if counts.top_base == 0 {
        f64::MAX
    } else if counts.top == counts.top_base {
        -f64::MAX
    } else {
        roofs.storage[bases[(index + counts.top - counts.top_base - 1).min(bases.len() - 1)]]
            .print_z
    };
    for &contact in contacts {
        let contact = &roofs.storage[contact];
        if contact.print_z < current.print_z {
            continue;
        }
        if contact.bottom_z - EPSILON > top_z {
            break;
        }
        let target = if contact.bottom_z - EPSILON > interface_z {
            &mut base
        } else {
            &mut interface
        };
        target.extend_from_slice(&contact.polygons);
    }
    (interface, base)
}

fn project_bottom(
    roofs: &RoofLayers,
    counts: &InterfaceCounts,
    bases: &[usize],
    contacts: &[usize],
    index: usize,
) -> (Vec<Polygon>, Vec<Polygon>) {
    let mut interface = Vec::new();
    let mut base = Vec::new();
    if counts.bottom == 0 {
        return (interface, base);
    }
    let current = &roofs.storage[bases[index]];
    let bottom_z =
        roofs.storage[bases[(index as i32 - counts.bottom as i32 + 1).max(0) as usize]].bottom_z;
    let interface_z = if counts.bottom_base == 0 {
        -f64::MAX
    } else if counts.bottom == counts.bottom_base {
        f64::MAX
    } else {
        roofs.storage
            [bases[(index as i32 - (counts.bottom - counts.bottom_base) as i32).max(0) as usize]]
            .bottom_z
    };
    for &contact in contacts {
        let contact = &roofs.storage[contact];
        if contact.print_z < bottom_z - EPSILON {
            continue;
        }
        if contact.print_z - EPSILON > current.bottom_z {
            break;
        }
        let target = if contact.print_z - EPSILON > interface_z {
            &mut interface
        } else {
            &mut base
        };
        target.extend_from_slice(&contact.polygons);
    }
    (interface, base)
}

fn same_layer(roofs: &RoofLayers, slots: &[usize], z: f64) -> Option<usize> {
    slots
        .iter()
        .copied()
        .find(|&index| roofs.storage[index].print_z > z - EPSILON)
        .filter(|&index| roofs.storage[index].print_z < z + EPSILON)
}

fn insert(
    roofs: &mut RoofLayers,
    base: usize,
    (bottom, top, existing): (Vec<Polygon>, Vec<Polygon>, Option<usize>),
    (subtract, kind): (Option<usize>, LayerType),
    (parameters, scale): (&ToolpathParameters, CoordinateScale),
) -> Result<Option<usize>, ClipperError> {
    let has_existing = existing.is_some_and(|index| !roofs.storage[index].polygons.is_empty());
    if bottom.is_empty() && top.is_empty() && !has_existing {
        return Ok(None);
    }
    let mut polygons = regularize(
        bottom,
        parameters.bottom_interface_density,
        parameters,
        scale,
    )?;
    polygons.extend(regularize(
        top,
        parameters.top_interface_density,
        parameters,
        scale,
    )?);
    polygons = intersection_polygons_paths(&polygons, &roofs.storage[base].polygons)?;
    if has_existing {
        let previous = std::mem::take(&mut roofs.storage[existing.unwrap()].polygons);
        polygons = union_polygons_paths_pair(&previous, &polygons)?;
    }
    if polygons.is_empty() {
        return Ok(None);
    }
    let source = &roofs.storage[base];
    let print_z = source.print_z;
    let bottom_z = source.bottom_z;
    let height = source.height;
    let index = existing.unwrap_or_else(|| {
        let index = roofs.storage.len();
        roofs.storage.push(GeneratorLayer {
            kind,
            print_z,
            bottom_z,
            height,
            polygons: Vec::new(),
        });
        index
    });
    roofs.storage[base].polygons =
        difference_polygons_paths(&roofs.storage[base].polygons, &polygons)?;
    if let Some(subtract) = subtract {
        polygons = difference_polygons_paths(&polygons, &roofs.storage[subtract].polygons)?;
    }
    let layer = &mut roofs.storage[index];
    layer.print_z = print_z;
    layer.bottom_z = bottom_z;
    layer.height = height;
    layer.polygons = polygons;
    Ok(Some(index))
}

fn regularize(
    polygons: Vec<Polygon>,
    density: f64,
    parameters: &ToolpathParameters,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    if polygons.is_empty() {
        return Ok(polygons);
    }
    let spacing = scale.scaled_delta(f64::from(parameters.interface_flow.spacing)) as i64;
    let distance = spacing as f64 * 1.5;
    let expanded = offset_paths(
        &polygons,
        (distance + spacing as f64 / density) as f32,
        JoinType::Square,
        0.0,
    )?;
    let closed = offset_paths(&expanded, -(distance as f32), JoinType::Square, 0.0)?;
    Ok(smooth_outward(closed, distance as i64, scale))
}

pub(super) fn indices(slots: &[Option<usize>]) -> Vec<usize> {
    slots.iter().flatten().copied().collect()
}

fn merge(
    roofs: &RoofLayers,
    mut generated: Vec<usize>,
    mut previous: Vec<usize>,
) -> Vec<Option<usize>> {
    generated.retain(|&index| !roofs.storage[index].polygons.is_empty());
    previous.retain(|&index| !roofs.storage[index].polygons.is_empty());
    let mut left = generated.into_iter().peekable();
    let mut right = previous.into_iter().peekable();
    let mut output = Vec::with_capacity(left.len() + right.len());
    while let (Some(&l), Some(&r)) = (left.peek(), right.peek()) {
        output.push(if roofs.storage[r].print_z < roofs.storage[l].print_z {
            right.next()
        } else {
            left.next()
        });
    }
    output.extend(left.map(Some));
    output.extend(right.map(Some));
    output
}
