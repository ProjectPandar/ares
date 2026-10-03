use super::super::{
    layer_region::PreparedLayerRegionPerimeterObject, types::PostPerimeterInputPrintObject,
};
use crate::geometry::{
    difference_ex_polygons_with_safety_offset, intersection_ex, union_expolygons,
};
use crate::{ProcessCounterboreHoleBridging, SliceError};

pub(in crate::project_slice::perimeters) fn apply(
    source: &PostPerimeterInputPrintObject,
    output: &mut PreparedLayerRegionPerimeterObject,
) -> Result<(), SliceError> {
    for group in &source.groups {
        if group.members.len() == 1 {
            continue;
        }
        let owner_slot = source.region_layer_record_index(group.owner);
        let owner = output.records[owner_slot].as_mut().unwrap();
        if owner.fill_surfaces.is_empty() {
            continue;
        }
        let generated = std::mem::take(&mut owner.fill_surfaces);
        let no_overlap = std::mem::take(&mut owner.fill_no_overlap_expolygons);
        let fill = generated
            .iter()
            .map(|surface| surface.as_parts().1.clone())
            .collect::<Vec<_>>();
        for &member in &group.members {
            let slot = source.region_layer_record_index(member);
            let slices = source
                .region_surfaces(member)
                .iter()
                .map(|surface| surface.as_parts().1.clone())
                .collect::<Vec<_>>();
            let record = output.records[slot].as_mut().unwrap();
            record.fill_expolygons = intersection_ex(&fill, &slices).map_err(geometry_error)?;
            record.fill_surfaces = record
                .fill_expolygons
                .iter()
                .map(|polygon| generated[0].clone_with_expolygon(polygon.clone()))
                .collect();
            record.fill_no_overlap_expolygons =
                intersection_ex(&slices, &no_overlap).map_err(geometry_error)?;
        }
        let input = source.records[owner_slot].as_ref().unwrap();
        if source.region_options(input).counterbore_hole_bridging
            != ProcessCounterboreHoleBridging::None
        {
            let slices = group
                .members
                .iter()
                .flat_map(|&member| source.region_surfaces(member))
                .flat_map(|surface| {
                    std::iter::once(surface.as_parts().1.contour())
                        .chain(surface.as_parts().1.holes())
                })
                .cloned()
                .collect::<Vec<_>>();
            let extra = difference_ex_polygons_with_safety_offset(&fill, &slices)
                .map_err(geometry_error)?;
            if !extra.is_empty() {
                let first_slot = source.region_layer_record_index(group.members[0]);
                let first = output.records[first_slot].as_mut().unwrap();
                first.fill_surfaces.extend(
                    extra
                        .iter()
                        .map(|polygon| generated[0].clone_with_expolygon(polygon.clone())),
                );
                first.fill_expolygons =
                    union_expolygons(&first.fill_expolygons, &extra).map_err(geometry_error)?;
            }
        }
    }
    Ok(())
}

fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("invalid compatible perimeter fill geometry".into())
}
