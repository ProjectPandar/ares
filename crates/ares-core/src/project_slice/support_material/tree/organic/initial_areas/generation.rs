//! Active `TreeSupport3D.cpp:1246-1456::generate_initial_areas`.

use std::collections::VecDeque;

use super::{InitialAreas, prepare_layer, raft_contact};
use crate::{
    ObjectOptions,
    geometry::{ClipperError, Coord, CoordinateScale, ExPolygon, FillRule, Polygon, union_ex},
    project_slice::support_material::tree::{
        model_volumes::TreeModelVolumes,
        organic::{
            interface_placer::InterfacePlacer,
            rich_interface_placer::RichInterfacePlacer,
            roof_sampler::{self, OverhangSample},
            support_element::SupportElement,
        },
    },
};

pub(in super::super) struct ObjectInputs<'a> {
    pub(in super::super) slices: &'a [Vec<ExPolygon>],
    pub(in super::super) overhangs: &'a [Vec<Polygon>],
    pub(in super::super) options: &'a ObjectOptions,
    pub(in super::super) scale: CoordinateScale,
}

pub(in super::super) fn generate(
    input: ObjectInputs<'_>,
    volumes: &mut TreeModelVolumes<'_>,
    interface: &mut InterfacePlacer<'_>,
    move_bounds: &mut [VecDeque<SupportElement>],
) -> Result<(), ClipperError> {
    let config = interface.config;
    let mesh = &config.mesh;
    let delta = config.z_distance_top_layers + 1;
    let connect_length = (mesh.support_line_width as f64 * 100.0 / mesh.top_rate
        + (2.0 * config.min_radius as f64 - 1.0 * mesh.support_line_width as f64).max(0.0))
        as Coord;
    let roof_layers = mesh.roof_layers as usize;
    let radius = config.min_radius as f64;
    let force_tip_to_roof = roof_layers > 0
        && (interface.slicing.zero_gap_interface_top
            || radius * radius * std::f64::consts::PI > mesh.minimum_roof_area);
    let first_support_layer = (config.raft_layers.len() as i32 - delta as i32).max(1) as usize;
    let num_support_layers = (input.slices.len() as i32 + config.raft_layers.len() as i32
        - delta as i32)
        .max(0) as usize;
    let raft_contact = raft_contact::generate(&input, interface)?;
    {
        let mut placer = RichInterfacePlacer::new(
            interface,
            volumes,
            force_tip_to_roof,
            move_bounds,
            input.scale,
        );
        for layer_index in first_support_layer..num_support_layers {
            let raw = &input.overhangs[layer_index + delta];
            if raw.is_empty() {
                continue;
            }
            let initial = prepare_layer(raw, layer_index, placer.volumes, config, input.scale)?;
            sample_areas(
                initial,
                (layer_index, connect_length),
                &mut placer,
                input.scale,
            )?;
        }
    }
    raft_contact::finalize(&input, raft_contact, interface, move_bounds)
}

fn sample_areas(
    initial: InitialAreas,
    (layer_index, connect_length): (usize, Coord),
    placer: &mut RichInterfacePlacer<'_, '_, '_>,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let roof_layers = placer.interface.config.mesh.roof_layers as usize;
    for (large_horizontal_roof, areas) in [(true, initial.roofs), (false, initial.regular)] {
        for component in union_ex(&areas, FillRule::NonZero)? {
            let (contour, holes) = component.into_parts();
            let mut polygons = Vec::with_capacity(holes.len() + 1);
            polygons.push(contour);
            polygons.extend(holes);
            roof_sampler::sample(
                OverhangSample {
                    polygons,
                    large_horizontal_roof,
                    layer_index,
                    roof_layers,
                    connect_length,
                },
                placer,
                scale,
            )?;
        }
    }
    Ok(())
}
