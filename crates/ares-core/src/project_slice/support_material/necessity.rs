//! `PrintObject::generate_support_material:966-986`, before its generation guard.
use super::{layer_input::LayerInput, tree};
use crate::{
    FloatOrPercent, PrintWarning, SliceError,
    project_slice::fill_entities::PreparedPostFillEntities,
};

pub(in crate::project_slice) fn check(
    prepared: &PreparedPostFillEntities,
    context: &mut crate::SlicerContext,
) -> Result<Vec<PrintWarning>, SliceError> {
    let horizontal = &prepared.predecessor.predecessor.predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    let settings = &traversal.resolved.views.full;
    let mut warnings = Vec::new();
    for source in &traversal.objects {
        let input = &source
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let (compensated, _) = input.as_parts();
        let (post_regions, slices) = compensated.as_parts();
        let plan = &post_regions.plan;
        let object = &traversal
            .resolved
            .objects
            .iter()
            .find(|resolved| resolved.source_object_index == plan.source_object_index)
            .expect("planned support object retains its resolved configuration")
            .object;
        if object.enable_support.0 || object.enforce_support_layers.0 > 0 {
            continue;
        }
        context.set_status(50, format_args!("Checking support necessity"), 0);
        // No no_check option exists in the current public request: source default false.
        // Neither validate_print=false nor raft-only suppresses this check.
        let layers = plan
            .layers
            .iter()
            .enumerate()
            .map(|(index, layer)| LayerInput {
                slices: &slices[index],
                regions: &[],
                height: layer.height,
            })
            .collect::<Vec<_>>();
        let first_nozzle = settings.project.print.nozzle_diameter.0[0].0;
        let width = match object.line_width {
            FloatOrPercent::Float(width) => width,
            FloatOrPercent::Percent(percent) => first_nozzle * percent.0 / 100.0,
        };
        if let Some(warning) = tree::is_support_necessary(
            object,
            &layers,
            width,
            traversal.scale,
            traversal.project.objects()[plan.source_object_index].name(),
        )
        .map_err(|error| SliceError::InvalidInput(format!("tree support geometry: {error:?}")))?
        {
            warnings.push(warning);
        }
    }
    Ok(warnings)
}
