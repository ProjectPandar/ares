//! Rendering-neutral data ported from AGPL-licensed OrcaSlicer `src/libvgcode` sources.

mod color_range;
pub mod convert;
mod extrusion_roles;
mod filament_legend;
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
pub use extrusion_roles::ExtrusionRoles;
pub use filament_legend::{FilamentColumn, FilamentLegend, FilamentRow, filament_legend};
pub use input_data::{ColorPrint, GCodeInputData};
pub use layers::Layers;
pub use moves_slider::MovesSlider;
pub use path_vertex::PathVertex;
pub use range::{Range, ViewRange};
pub use sequential_view::{
    ActualSpeedItem, ActualSpeedProfile, MarkerTime, SequentialView, ToolMarker,
};
pub use settings::Settings;
pub use types::{
    AABox, Color, ColorRangeType, DEFAULT_TRAVELS_RADIUS_MM, DEFAULT_WIPES_RADIUS_MM, DUMMY_COLOR,
    GCodeExtrusionRole, Interval, MAX_TRAVELS_RADIUS_MM, MAX_WIPES_RADIUS_MM,
    MIN_TRAVELS_RADIUS_MM, MIN_WIPES_RADIUS_MM, Mat4x4, MoveType, OptionType, Palette, TimeMode,
    Vec3, ViewType, lerp_color, move_type_to_option,
};
pub use viewer::{
    ColorPrintLegend, ColorPrintRange, CustomGCodeOverviewRow, DEFAULT_EXTRUSION_ROLES_COLORS,
    DEFAULT_OPTIONS_COLORS, ExtruderColorPrints, MissingColorVolume, PartialTime, PartialTimeKind,
    PrintedColor, ViewerImpl, color_print_legend, custom_gcode_overview,
};
