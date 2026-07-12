//! Rendering-neutral data ported from AGPL-licensed OrcaSlicer `src/libvgcode` sources.

mod input_data;
mod layers;
mod path_vertex;
mod range;
mod types;

pub use input_data::{ColorPrint, GCodeInputData};
pub use layers::Layers;
pub use path_vertex::PathVertex;
pub use range::{Range, ViewRange};
pub use types::{
    AABox, Color, ColorRangeType, DEFAULT_TRAVELS_RADIUS_MM, DEFAULT_WIPES_RADIUS_MM, DUMMY_COLOR,
    GCodeExtrusionRole, Interval, MAX_TRAVELS_RADIUS_MM, MAX_WIPES_RADIUS_MM,
    MIN_TRAVELS_RADIUS_MM, MIN_WIPES_RADIUS_MM, Mat4x4, MoveType, OptionType, Palette, TimeMode,
    Vec3, ViewType, lerp_color, move_type_to_option,
};
