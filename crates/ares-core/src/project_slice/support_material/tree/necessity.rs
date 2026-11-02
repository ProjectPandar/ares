//! `PrintObject::is_support_necessary:5289-5303`; check-only return at TreeSupport:1083.
use super::overhangs::SmartOverhangs;
use crate::{
    ObjectOptions, PrintWarning, SlicingNotificationType, WarningLevel,
    geometry::{ClipperError, CoordinateScale},
    project_slice::support_material::layer_input::LayerInput,
};

pub(in crate::project_slice::support_material) fn is_support_necessary(
    object: &ObjectOptions,
    layers: &[LayerInput<'_>],
    extrusion_width: f64,
    scale: CoordinateScale,
    name: &str,
) -> Result<Option<PrintWarning>, ClipperError> {
    let mut overhangs = SmartOverhangs::new(object, layers, extrusion_width, scale)?;
    // The effective TreeAuto override is invocation-local, not a config mutation.
    overhangs.automatic = true;
    let start = web_time::Instant::now();
    for layer in 0..layers.len() {
        overhangs.detect_layer(layer, || start.elapsed().as_secs_f64())?;
    }
    let feature = if overhangs.has_sharp_tails {
        "floating regions"
    } else if overhangs.has_cantilever
        && overhangs.max_cantilever_distance > scale.scaled_delta(6.0)
    {
        "a floating cantilever"
    } else {
        return Ok(None);
    };
    // No propagation/finalization or installed support layers: scratch drops here.
    Ok(Some(PrintWarning {
        level: WarningLevel::NonCritical,
        message_type: SlicingNotificationType::NeedSupportOn,
        message: format!(
            "It seems object {name} has {feature}. Please re-orient the object or enable support generation."
        ),
    }))
}
