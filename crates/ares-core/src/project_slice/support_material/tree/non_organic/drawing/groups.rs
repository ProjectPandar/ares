//! `Layer.hpp::AreaGroup` and `TreeSupport.cpp:2365-2444` group/adhesion identity.

use super::areas::LayerAreas;
use crate::geometry::{BoundingBox, ClipperError, CoordinateScale, ExPolygon, union_expolygons};
use crate::project_slice::support_material::overhangs::overlaps;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(in super::super) enum AreaKind {
    Base,
    Roof,
    Floor,
    RoofFirst,
}

pub(in super::super) struct AreaGroup {
    pub(in super::super) kind: AreaKind,
    pub(in super::super) index: usize,
    pub(in super::super) interface_id: usize,
    pub(in super::super) interface_as_base: bool,
    pub(in super::super) need_infill: bool,
    pub(in super::super) need_extra_wall: bool,
}

pub(super) fn build_groups(
    areas: &mut LayerAreas,
    extra_wall_areas: &[ExPolygon],
    collect_slices: bool,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let mut groups = Vec::with_capacity(
        areas.base.len() + areas.roof.len() + areas.floor.len() + areas.roof_first.len(),
    );
    for (kind, polygons) in [
        (AreaKind::Base, &mut areas.base),
        (AreaKind::Roof, &mut areas.roof),
        (AreaKind::Floor, &mut areas.floor),
        (AreaKind::RoofFirst, &mut areas.roof_first),
    ] {
        for (index, polygon) in polygons.iter().enumerate() {
            let region = std::slice::from_ref(polygon);
            let need_infill = kind == AreaKind::Base && overlaps(region, &areas.polygons)?;
            let need_extra_wall = kind == AreaKind::Base
                && !need_infill
                && !extra_wall_areas.is_empty()
                && overlaps(region, extra_wall_areas)?;
            groups.push(AreaGroup {
                kind,
                index,
                interface_id: 0,
                interface_as_base: kind == AreaKind::Floor && areas.floor_interface_as_base,
                need_infill,
                need_extra_wall,
            });
        }
        remove_small_holes(polygons, scale);
        if collect_slices {
            areas.slices.extend_from_slice(polygons);
        }
    }
    areas.groups = groups;
    areas.slices = union_expolygons(&areas.slices, &[])?;
    areas.slice_bounds = areas
        .slices
        .iter()
        .map(|polygon| BoundingBox::from_expolygon(polygon).expect("support slice has a contour"))
        .collect();
    Ok(())
}

fn remove_small_holes(polygons: &mut [ExPolygon], scale: CoordinateScale) {
    for polygon in polygons {
        polygon.holes_mut().retain(|hole| {
            let bounds = BoundingBox::from_polygon(hole).expect("support hole has a contour");
            let width = bounds.max().x() - bounds.min().x();
            let height = bounds.max().y() - bounds.min().y();
            !((width as f64) < scale.scaled_delta(2.0) && (height as f64) < scale.scaled_delta(2.0))
        });
    }
}

pub(super) fn sequence_interfaces(
    layers: &mut [LayerAreas],
    interlaced: bool,
    top_base_layers: usize,
) {
    let mut roof_id = 0;
    let mut floor_id = 0;
    for layer in layers {
        let mut roof = false;
        let mut floor = false;
        for group in &mut layer.groups {
            let (has_roof, has_floor) =
                assign_interface(group, roof_id, floor_id, interlaced, top_base_layers);
            roof |= has_roof;
            floor |= has_floor;
        }
        roof_id += usize::from(roof);
        floor_id += usize::from(floor);
    }
}

fn assign_interface(
    group: &mut AreaGroup,
    roof_id: usize,
    floor_id: usize,
    interlaced: bool,
    top_base_layers: usize,
) -> (bool, bool) {
    match group.kind {
        AreaKind::Roof | AreaKind::RoofFirst => {
            if interlaced {
                group.interface_id = roof_id;
            }
            group.interface_as_base = top_base_layers > 0 && roof_id < top_base_layers;
            (true, false)
        }
        AreaKind::Floor => {
            if interlaced {
                group.interface_id = floor_id;
            }
            (false, true)
        }
        AreaKind::Base => (false, false),
    }
}
