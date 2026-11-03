//! `Print.cpp:3444–3537`: Type1 planning input, consumed by `generate_new`.
mod blocks;
mod data;
mod dynamic;
mod emit;
mod templates;
mod travel;
pub(super) use emit::append;
pub(super) use result::Metadata;
mod category_depths;
mod filament;
mod finish_block;
mod finish_layer;
mod generate;
mod generator;
mod grid;
mod nozzle_change;
mod outer_wall;
mod plan;
mod planning;
mod result;
mod support_wall;
mod tool_change;
mod wall_filament;
mod wall_gaps;
mod wipe;
mod writer;

pub(super) use planning::{Planned, prepare};
