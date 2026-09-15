//! The per-append emission context (`layers::Context`).

use crate::project_slice::gcode_emit::layers::GenerationMetadata;
use crate::project_slice::gcode_emit::value::Value;
use crate::project_slice::gcode_emit::{brim, footprint, skirt};

pub(crate) struct Context<'a> {
    pub(crate) metadata: GenerationMetadata,
    pub(crate) first_layer_bounds: Option<footprint::FirstLayerBounds>,
    pub(crate) start_position: Option<Value>,
    pub(crate) bed_cache: i32,
    pub(crate) extruder_offset: (f64, f64),
    pub(crate) brim: &'a Option<brim::BrimPlan>,
    pub(crate) skirt: &'a Option<skirt::SkirtPlan>,
}
