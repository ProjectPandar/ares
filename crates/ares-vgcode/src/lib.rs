//! Rendering-neutral data ported from AGPL-licensed OrcaSlicer `src/libvgcode` sources.

mod color_range;
pub mod convert;
mod input_data;
mod layers;
mod moves_slider;
mod path_vertex;
mod range;
mod sequential_view;
mod settings;
mod types;
mod viewer;

pub use color_range::{ColorRange, DEFAULT_RANGES_COLORS};
pub use input_data::{ColorPrint, GCodeInputData};
pub use layers::Layers;
pub use moves_slider::MovesSlider;
pub use path_vertex::PathVertex;
pub use range::{Range, ViewRange};
pub use sequential_view::{SequentialView, ToolMarker};
pub use settings::Settings;
pub use types::{
    AABox, Color, ColorRangeType, DEFAULT_TRAVELS_RADIUS_MM, DEFAULT_WIPES_RADIUS_MM, DUMMY_COLOR,
    GCodeExtrusionRole, Interval, MAX_TRAVELS_RADIUS_MM, MAX_WIPES_RADIUS_MM,
    MIN_TRAVELS_RADIUS_MM, MIN_WIPES_RADIUS_MM, Mat4x4, MoveType, OptionType, Palette, TimeMode,
    Vec3, ViewType, lerp_color, move_type_to_option,
};
pub use viewer::{DEFAULT_EXTRUSION_ROLES_COLORS, DEFAULT_OPTIONS_COLORS, ViewerImpl};
