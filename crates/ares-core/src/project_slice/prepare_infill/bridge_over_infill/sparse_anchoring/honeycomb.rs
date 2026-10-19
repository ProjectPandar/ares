use crate::{
    SliceError,
    fill::honeycomb::{HoneycombFillParams, fill_honeycomb},
    geometry::{JoinType, Polyline, offset_expolygon},
    project_slice::{
        group_fills::SurfaceFill,
        perimeters::classic::traversal::types::PreparedPostClassicTraversal,
    },
};

pub(super) fn append(
    fill: SurfaceFill,
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    layer_index: usize,
    result: &mut Vec<Polyline>,
) -> Result<(), SliceError> {
    // Fill.cpp:1444-1446: use the anchoring fill's printing region.
    let (resolved_object, _) = traversal.print_object_config(object_index);
    let region = &resolved_object.all_regions[fill.region_id];
    let params = HoneycombFillParams {
        dont_alternate_fill_direction: region.zaa_enabled.0
            && region.zaa_dont_alternate_fill_direction.0,
        spacing: fill.params.spacing,
        density: (0.01 * f64::from(fill.params.density)) as f32,
        multiline: fill.params.multiline,
        angle: fill.params.angle,
        layer_index,
        thickness_layers: fill.representative.thickness_layers.max(1),
        fixed_angle: fill.params.fixed_angle,
        bridge_angle: fill.params.bridge.then_some(fill.params.bridge_angle),
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
    };
    let delta = (-0.5 * fill.params.spacing / traversal.scale.factor()) as f32;
    for expolygon in fill.expolygons {
        let domains = offset_expolygon(&expolygon, delta, JoinType::Miter, 3.0)
            .map_err(super::super::transaction::geometry_error)?;
        for domain in domains {
            let polylines = fill_honeycomb(&domain, params, traversal.scale);
            result.extend(polylines.map_err(super::super::transaction::geometry_error)?);
        }
    }
    Ok(())
}
