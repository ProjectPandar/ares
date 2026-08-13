use super::{
    ExpansionZone,
    expand_bridges_detect_orientations::expand_bridges_detect_orientations,
    expand_merge::expand_merge_surfaces,
    parameters::{ProcessExternalSurfacesConfig, derive},
};
use crate::{
    geometry::{ClipperError, ExPolygon, RegionExpansionParameters, union_expolygons},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(in crate::project_slice) fn process_external_surfaces(
    surfaces: &mut Vec<RegionSurface>,
    config: ProcessExternalSurfacesConfig,
) -> Result<(), ClipperError> {
    let parameters = derive(config);
    let mut layer_thickness = -1.0;
    let shells = extract_and_union(
        surfaces,
        RegionSurfaceKind::InternalSolid,
        &mut layer_thickness,
    )?;
    let sparse = extract_and_union(surfaces, RegionSurfaceKind::Internal, &mut layer_thickness)?;
    let top = extract_and_union(surfaces, RegionSurfaceKind::Top, &mut layer_thickness)?;

    let solid_parameters = RegionExpansionParameters::build(
        parameters.expansion_bottom_bridge,
        parameters.expansion_step,
        5,
        config.scale,
    );
    let sparse_parameters = RegionExpansionParameters::build(
        parameters.expansion_min,
        parameters.expansion_step,
        5,
        config.scale,
    );
    let mut zones = vec![
        ExpansionZone::new(shells, solid_parameters),
        ExpansionZone::new(sparse, sparse_parameters),
        ExpansionZone::new(top, solid_parameters),
    ];

    let custom_angle = std::f64::consts::PI * config.bridge_angle_degrees / 180.0;
    let mut bridges = if config.bridge_angle_degrees > 0.0 && !config.relative_bridge_angle {
        expand_merge_surfaces(
            surfaces,
            RegionSurfaceKind::BottomBridge,
            &mut zones,
            parameters.closing_radius,
            custom_angle + config.model_rotation_radians,
            config.scale,
        )?
    } else {
        expand_bridges_detect_orientations(
            surfaces,
            &mut zones,
            parameters.closing_radius,
            config.scale,
        )?
    };
    if config.bridge_angle_degrees > 0.0 && config.relative_bridge_angle {
        for bridge in &mut bridges {
            let angle = bridge.as_parts().4;
            if angle >= 0.0 {
                bridge.set_bridge_angle(angle + custom_angle);
            }
        }
    }

    surfaces.retain(|surface| surface.as_parts().0 != RegionSurfaceKind::Top);
    surfaces.reserve(zones[2].expolygons.len());
    append_surfaces(
        surfaces,
        std::mem::take(&mut zones[2].expolygons),
        RegionSurfaceKind::Top,
        layer_thickness,
    );
    zones.pop();

    zones[0].parameters = RegionExpansionParameters::build(
        parameters.expansion_bottom,
        parameters.expansion_step,
        5,
        config.scale,
    );
    let bottoms = expand_merge_surfaces(
        surfaces,
        RegionSurfaceKind::Bottom,
        &mut zones,
        parameters.closing_radius,
        -1.0,
        config.scale,
    )?;
    zones[0].parameters = RegionExpansionParameters::build(
        parameters.expansion_top,
        parameters.expansion_step,
        5,
        config.scale,
    );
    let tops = expand_merge_surfaces(
        surfaces,
        RegionSurfaceKind::Top,
        &mut zones,
        parameters.closing_radius,
        -1.0,
        config.scale,
    )?;

    if !config.spiral_mode && config.sparse_infill_density_percent > 0.0 {
        let small = zones[1]
            .expolygons
            .extract_if(.., |expolygon| {
                expolygon.area() <= parameters.minimum_sparse_area
            })
            .collect::<Vec<_>>();
        if !small.is_empty() {
            zones[0].expolygons = union_expolygons(&zones[0].expolygons, &small)?;
        }
    }

    let surface_count = zones[0].expolygons.len()
        + zones[1].expolygons.len()
        + bridges.len()
        + bottoms.len()
        + tops.len();
    surfaces.clear();
    surfaces.reserve(surface_count);
    append_surfaces(
        surfaces,
        std::mem::take(&mut zones[0].expolygons),
        RegionSurfaceKind::InternalSolid,
        layer_thickness,
    );
    append_surfaces(
        surfaces,
        std::mem::take(&mut zones[1].expolygons),
        RegionSurfaceKind::Internal,
        layer_thickness,
    );
    surfaces.extend(bridges);
    surfaces.extend(bottoms);
    surfaces.extend(tops);
    Ok(())
}

fn extract_and_union(
    surfaces: &mut [RegionSurface],
    kind: RegionSurfaceKind,
    layer_thickness: &mut f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut extracted = Vec::with_capacity(
        surfaces
            .iter()
            .filter(|surface| surface.as_parts().0 == kind)
            .count(),
    );
    for surface in surfaces {
        if surface.as_parts().0 == kind {
            *layer_thickness = surface.as_parts().2;
            extracted.push(surface.take_expolygon());
        }
    }
    union_expolygons(&[], &extracted)
}

fn append_surfaces(
    surfaces: &mut Vec<RegionSurface>,
    expolygons: Vec<ExPolygon>,
    kind: RegionSurfaceKind,
    thickness: f64,
) {
    for expolygon in expolygons {
        let mut surface = RegionSurface::new(kind, expolygon);
        surface.set_thickness(thickness);
        surfaces.push(surface);
    }
}
