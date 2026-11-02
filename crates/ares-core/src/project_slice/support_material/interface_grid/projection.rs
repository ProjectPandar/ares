use super::{GridInterfacesInput, IntermediateLayer};
use crate::geometry::Polygon;
use crate::project_slice::support_material::layer_index::idx_higher_or_equal;

const EPSILON: f64 = 1e-4;

pub(super) fn top(
    input: &GridInterfacesInput<'_>,
    bases: &[IntermediateLayer],
    index: usize,
    first: &mut i32,
) -> (Vec<Polygon>, Vec<Polygon>) {
    let counts = input.counts;
    let mut interface = Vec::new();
    let mut base = Vec::new();
    if counts.top == 0 {
        return (interface, base);
    }
    let top_z = bases[(index + counts.top - 1).min(bases.len() - 1)].print_z;
    let interface_z = if counts.top_base == 0 {
        f64::MAX
    } else if counts.top - counts.top_base == 0 {
        -f64::MAX
    } else {
        bases[(index + counts.top - counts.top_base - 1).min(bases.len() - 1)].print_z
    };
    let start = idx_higher_or_equal(input.tops, first, |top| top.print_z >= bases[index].print_z);
    for top in &input.tops[start as usize..] {
        if top.bottom_z - EPSILON > top_z {
            break;
        }
        let target = if top.bottom_z - EPSILON > interface_z {
            &mut base
        } else {
            &mut interface
        };
        target.extend_from_slice(if input.style == crate::ProcessSupportStyle::Snug {
            top.overhang_polygons.as_deref().unwrap()
        } else {
            &top.polygons
        });
    }
    (interface, base)
}

pub(super) fn bottom(
    input: &GridInterfacesInput<'_>,
    bases: &[IntermediateLayer],
    index: usize,
    first: &mut i32,
) -> (Vec<Polygon>, Vec<Polygon>) {
    let counts = input.counts;
    let mut interface = Vec::new();
    let mut base = Vec::new();
    if counts.bottom == 0 {
        return (interface, base);
    }
    let bottom_z = bases[(index as i32 - counts.bottom as i32 + 1).max(0) as usize].bottom_z;
    let interface_z = if counts.bottom_base == 0 {
        -f64::MAX
    } else if counts.bottom - counts.bottom_base == 0 {
        f64::MAX
    } else {
        bases[(index as i32 - (counts.bottom - counts.bottom_base) as i32).max(0) as usize].bottom_z
    };
    let start = idx_higher_or_equal(input.bottoms, first, |bottom| {
        bottom.print_z >= bottom_z - EPSILON
    });
    for bottom in &input.bottoms[start as usize..] {
        if bottom.print_z - EPSILON > bases[index].bottom_z {
            break;
        }
        let target = if bottom.print_z - EPSILON > interface_z {
            &mut interface
        } else {
            &mut base
        };
        target.extend_from_slice(&bottom.polygons);
    }
    (interface, base)
}
