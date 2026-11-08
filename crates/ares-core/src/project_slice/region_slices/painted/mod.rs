//! Consumed rewrite of MultiMaterialSegmentation.cpp1955–2207.
mod apply;
mod fuzzy;
pub(in crate::project_slice) use fuzzy::apply_project as apply_fuzzy_project;
mod contour_color;
mod cut;
mod graph;
mod math;
mod merge;
mod painted_lines;
mod preprocess;
mod projection;
mod projection_visitor;
mod slabs;
mod top_bottom;

use crate::project_slice::{PreparedPostRegions, centering};
use crate::{
    SliceError,
    geometry::{ClipperError, ExPolygon},
};
pub(in crate::project_slice) use apply::ObjectRegions;

fn clip_error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("painted geometry failed: {error:?}"))
}
fn sides(
    input: &projection::Input<'_>,
    layers: &[preprocess::Layer],
) -> Result<Vec<Vec<Vec<ExPolygon>>>, SliceError> {
    let projected = projection::project(input, layers)?;
    let mut result = Vec::with_capacity(layers.len());
    let epsilon = input.scale.scaled_delta(f64::from(0.1_f32)) as i64 as f64;
    for (layer, lines) in layers.iter().zip(projected) {
        let mut colors = vec![Vec::new(); input.states];
        if let Some(grid) = &layer.grid
            && !lines.is_empty()
        {
            let painted = painted_lines::post_process(grid.contours(), lines, epsilon);
            let contours = contour_color::contours(grid.contours(), &painted, input.scale);
            if contour_color::single_color(&contours) {
                colors[contours[0][0].color] = layer.input.clone();
            } else {
                colors = graph::segment(&contours, input.states, input.scale)?;
            }
        }
        result.push(colors);
    }
    Ok(result)
}
pub(in crate::project_slice) fn apply_project(
    prepared: &mut PreparedPostRegions,
) -> Result<(), SliceError> {
    let full = &prepared.resolved.views.full;
    let states = full.filament.gcode.filament_colour.0.len() + 1;
    let extruders = full.filament.gcode.filament_diameter.0.len();
    let nozzle = full.project.print.nozzle_diameter.0[0].0;
    for (object, metadata) in prepared.objects.iter_mut().zip(&prepared.painting) {
        if !metadata.active {
            continue;
        }
        let resolved = prepared
            .resolved
            .objects
            .iter()
            .find(|resolved| resolved.source_object_index == object.plan.source_object_index)
            .unwrap();
        let source = &prepared.project.objects()[resolved.source_object_index];
        let planes = object
            .plan
            .layers
            .iter()
            .map(|layer| layer.slice_z)
            .collect::<Vec<_>>();
        let heights = object
            .plan
            .layers
            .iter()
            .map(|layer| layer.height as f32)
            .collect::<Vec<_>>();
        let transform = &resolved.print_objects[object.plan.transform_index];
        let input = projection::Input {
            painting: projection::Painting::Mmu,
            source,
            transform: transform.transform.without_xy_translation(),
            center: centering::center(source, transform, prepared.scale)?,
            planes: &planes,
            states,
            scale: prepared.scale,
        };
        let layers = preprocess::prepare(object, prepared.scale).map_err(clip_error)?;
        let mut side = sides(&input, &layers)?;
        // MMS2199–2201 casts option doubles before2169–2170 scales the float millimeters.
        let width = resolved.object.mmu_segmented_region_max_width.0 as f32;
        let depth = resolved.object.mmu_segmented_region_interlocking_depth.0 as f32;
        if (width > 0.0 || depth > 0.0) && !resolved.object.interlocking_beam.0 {
            cut::cut_segmented_layers(
                &layers,
                &mut side,
                prepared.scale.scaled_delta(f64::from(width)) as f32,
                prepared.scale.scaled_delta(f64::from(depth)) as f32,
            )
            .map_err(clip_error)?;
        }
        let caps = top_bottom::segment(
            &input,
            &layers,
            (&object.regions, &prepared.resolved.views.full),
            &heights,
            nozzle,
        )?;
        let segmentation = merge::merge(side, caps, prepared.scale).map_err(clip_error)?;
        apply::apply(object, metadata, segmentation, extruders, prepared.scale)?;
    }
    Ok(())
}
