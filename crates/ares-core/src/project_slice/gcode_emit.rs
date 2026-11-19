use crate::project_slice::{
    island_print_order::PreparedPostIslandPrintOrder,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};
use crate::{GenerationMetadata, SliceError};

mod adaptive_pa;
mod brim;
mod chamber;
mod conflict;
mod emit_helpers;
use emit_helpers::{append_layer_end_timelapse, layer_boundary_slices_rc, trailing_gcode_xy};
mod cooling;
pub(crate) mod expression;
mod fan_chunks;
mod fan_mover;
mod file_start;
mod finish;
pub(super) mod footprint;
mod header;
mod klipper_acceleration;
mod layer_gcode;
mod layers;
pub(crate) mod lexer;
mod machine;
pub(in crate::project_slice) mod motion;
mod object;
mod offset;
mod placeholders;
mod preamble;
mod pressure_advance;
pub(crate) mod processor;
mod retained_output;
mod skirt;
mod small_area;
mod spiral_vase;
mod tags;
mod temperature;
mod template;
mod template_failure;

mod timelapse;
mod timelapse_pos;
mod tool_ordering;
mod tool_transition;

mod orchestration;
mod orchestration_finish;
mod orchestration_header;

pub(crate) mod value;
mod wipe_tower;
pub(super) use orchestration::emit;
pub(crate) use wipe_tower::estimate as wipe_tower_estimate;

pub(in crate::project_slice) use emit_helpers::format_processor_float;
