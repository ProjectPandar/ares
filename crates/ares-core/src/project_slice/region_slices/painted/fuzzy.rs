//! MultiMaterialSegmentation.cpp2211–2226: two states, external-flow-width cut, no caps.
use super::{apply, clip_error, cut, merge, preprocess, projection, sides};
use crate::{
    SliceError,
    project_slice::{PreparedPostRegions, centering, perimeters::flow},
};

pub(in crate::project_slice) fn apply_project(
    prepared: &mut PreparedPostRegions,
) -> Result<(), SliceError> {
    let nozzles = &prepared.resolved.views.full.project.print.nozzle_diameter;
    for (object, metadata) in prepared.objects.iter_mut().zip(&prepared.painting) {
        if metadata.fuzzy.iter().all(Vec::is_empty) {
            continue;
        }
        let resolved = prepared
            .resolved
            .objects
            .iter()
            .find(|resolved| resolved.source_object_index == object.plan.source_object_index)
            .unwrap();
        let source = &prepared.project.objects()[resolved.source_object_index];
        let transform = &resolved.print_objects[object.plan.transform_index];
        let planes = object
            .plan
            .layers
            .iter()
            .map(|layer| layer.slice_z)
            .collect::<Vec<_>>();
        let input = projection::Input {
            painting: projection::Painting::Fuzzy,
            source,
            transform: transform.transform.without_xy_translation(),
            center: centering::center(source, transform, prepared.scale)?,
            planes: &planes,
            states: 2,
            scale: prepared.scale,
        };
        let layers = preprocess::prepare(object, prepared.scale).map_err(clip_error)?;
        let mut side = sides(&input, &layers)?;
        let mut width = 0.0_f32;
        for region in &object.regions {
            width = width.max(
                flow::resolve_nominal_external_perimeter_flow(
                    &resolved.object,
                    &region.options,
                    nozzles,
                )?
                .width,
            );
        }
        if width > 0.0 {
            cut::cut_segmented_layers(
                &layers,
                &mut side,
                prepared.scale.scaled_delta(f64::from(width)) as f32,
                0.0,
            )
            .map_err(clip_error)?;
        }
        let segmentation =
            merge::merge(side, Vec::new(), &layers, prepared.scale).map_err(clip_error)?;
        apply::fuzzy::apply(object, metadata, segmentation, prepared.scale)?;
    }
    Ok(())
}
