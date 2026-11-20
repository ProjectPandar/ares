// Ports the color part of AGPL-licensed OrcaSlicer `src/libvgcode/src/ViewerImpl.cpp`:
// default colors (`:283-317`), `update_colors` (`:1261-1287`), `set_view_type`
// (`:1313-1317`), `update_colors_texture` (`:1227-1245`), `get_vertex_color` (`:1467-1553`), `get_color_range`
// (`:1599-1621`) and `update_color_ranges` (`:1817-1878`).

use super::ViewerImpl;
use crate::{
    Color, ColorRange, ColorRangeType, DUMMY_COLOR, GCodeExtrusionRole, MoveType, OptionType,
    PathVertex, ViewType, color_range::round_to_bin, move_type_to_option,
};

/// `DEFAULT_EXTRUSION_ROLES_COLORS` (`ViewerImpl.cpp:283-305`).
pub const DEFAULT_EXTRUSION_ROLES_COLORS: [Color; GCodeExtrusionRole::COUNT] = [
    [230, 179, 179],
    [255, 230, 77],
    [255, 125, 56],
    [31, 31, 255],
    [176, 48, 41],
    [150, 84, 204],
    [240, 64, 64],
    [255, 140, 105],
    [77, 128, 186],
    [255, 255, 255],
    [0, 135, 110],
    [0, 255, 0],
    [0, 128, 0],
    [179, 227, 171],
    [94, 209, 148],
    [102, 92, 199],
    [77, 128, 186],
    [0, 59, 110],
    [0, 64, 0],
    [128, 128, 128],
];

/// `DEFAULT_OPTIONS_COLORS` (`ViewerImpl.cpp:307-317`).
pub const DEFAULT_OPTIONS_COLORS: [Color; OptionType::COUNT] = [
    [56, 72, 155],
    [255, 255, 0],
    [205, 34, 214],
    [73, 173, 207],
    [230, 230, 230],
    [193, 190, 99],
    [218, 148, 139],
    [82, 240, 131],
    [226, 210, 67],
];

/// `ViewerImpl` color range members (`ViewerImpl.hpp:285-302`).
#[derive(Clone, Debug)]
pub(super) struct ColorRanges {
    height: ColorRange,
    width: ColorRange,
    speed: ColorRange,
    actual_speed: ColorRange,
    fan_speed: ColorRange,
    temperature: ColorRange,
    pressure_advance: ColorRange,
    acceleration: ColorRange,
    jerk: ColorRange,
    volumetric_rate: ColorRange,
    actual_volumetric_rate: ColorRange,
    /// Linear, logarithmic.
    layer_time: [ColorRange; ColorRangeType::COUNT],
}

impl Default for ColorRanges {
    fn default() -> Self {
        let linear = ColorRange::new(ColorRangeType::Linear);
        Self {
            height: linear.clone(),
            width: linear.clone(),
            speed: linear.clone(),
            actual_speed: linear.clone(),
            fan_speed: linear.clone(),
            temperature: linear.clone(),
            pressure_advance: linear.clone(),
            acceleration: linear.clone(),
            jerk: linear.clone(),
            volumetric_rate: linear.clone(),
            actual_volumetric_rate: linear.clone(),
            layer_time: [linear, ColorRange::new(ColorRangeType::Logarithmic)],
        }
    }
}

impl ColorRanges {
    fn reset(&mut self) {
        let [linear, log] = &mut self.layer_time;
        for range in [
            &mut self.width,
            &mut self.height,
            &mut self.speed,
            &mut self.actual_speed,
            &mut self.fan_speed,
            &mut self.temperature,
            &mut self.pressure_advance,
            &mut self.acceleration,
            &mut self.jerk,
            &mut self.volumetric_rate,
            &mut self.actual_volumetric_rate,
            linear,
            log,
        ] {
            range.reset();
        }
    }

    /// The extrusion branch of `update_color_ranges` (`ViewerImpl.cpp:1845-1857`).
    fn update_extrusion(&mut self, v: &PathVertex, custom_visible: bool) {
        self.height.update(round_to_bin(v.height));
        if !v.is_custom_gcode() || custom_visible {
            self.width.update(round_to_bin(v.width));
            self.volumetric_rate
                .update(round_to_bin(v.volumetric_rate()));
            self.actual_volumetric_rate
                .update(round_to_bin(v.actual_volumetric_rate()));
        }
        self.fan_speed.update(round_to_bin(v.fan_speed));
        self.temperature.update(round_to_bin(v.temperature));
        if v.pressure_advance >= 0.0 {
            self.pressure_advance.update(v.pressure_advance);
        }
    }
}

fn option_color(move_type: MoveType) -> Color {
    move_type_to_option(move_type)
        .map_or(DUMMY_COLOR, |option| DEFAULT_OPTIONS_COLORS[option.index()])
}

impl ViewerImpl {
    pub fn get_view_type(&self) -> ViewType {
        self.settings.view_type
    }

    pub fn set_view_type(&mut self, view_type: ViewType) {
        self.settings.view_type = view_type;
        self.settings.update_colors = true;
    }

    /// Per-vertex colors for the current view, recomputed as `render` does
    /// when `update_colors` is set (`ViewerImpl.cpp:1297-1298`); `load` fills them
    /// eagerly (`ViewerImpl.cpp:1127`), here on first use.
    pub fn colors(&mut self) -> &[Color] {
        if self.settings.update_colors {
            self.update_colors();
        }
        &self.vertices_colors
    }

    /// `update_colors_texture` (`ViewerImpl.cpp:1227-1245`): `colors`, dark grey
    /// below the top layer of a top-layer-only view while the visible range ends
    /// before the full range.
    pub fn texture_colors(&mut self) -> Vec<Color> {
        self.texture_colors_range(0..self.vertices.len())
    }

    pub fn texture_colors_range(&mut self, range: std::ops::Range<usize>) -> Vec<Color> {
        if self.settings.update_colors {
            self.update_colors();
        }
        let top_layer_id = if self.settings.top_layer_only_view_range {
            self.layers.get_view_range()[1]
        } else {
            0
        };
        let color_top_layer_only =
            self.view_range.get_full()[1] != self.view_range.get_visible()[1];
        let enabled_first = self.view_range.get_enabled()[0];
        let spiral_vase_mode = self.settings.spiral_vase_mode;
        range
            .clone()
            .zip(
                self.vertices[range.clone()]
                    .iter()
                    .zip(&self.vertices_colors[range]),
            )
            .map(|(i, (v, &color))| {
                if color_top_layer_only
                    && (v.layer_id as usize) < top_layer_id
                    && (!spiral_vase_mode || i != enabled_first)
                {
                    DUMMY_COLOR
                } else {
                    color
                }
            })
            .collect()
    }

    /// `update_colors` (`ViewerImpl.cpp:1261-1287`); tool colors past the
    /// palette read as `DUMMY_COLOR` instead of padding it.
    fn update_colors(&mut self) {
        self.update_color_ranges();
        self.vertices_colors = self
            .vertices
            .iter()
            .map(|v| self.get_vertex_color(v))
            .collect();
        self.settings.update_colors = false;
    }

    /// `get_vertex_color` (`ViewerImpl.cpp:1467-1553`).
    pub fn get_vertex_color(&self, v: &PathVertex) -> Color {
        if v.move_type == MoveType::Noop {
            return DUMMY_COLOR;
        }
        let view_type = self.settings.view_type;
        let speed_view = matches!(
            view_type,
            ViewType::Speed | ViewType::ActualSpeed | ViewType::Acceleration | ViewType::Jerk
        );
        if (v.is_wipe() && !speed_view) || v.is_option() {
            return option_color(v.move_type);
        }
        let ranges = &self.ranges;
        let extrusion = |range: &ColorRange, value: f32| {
            if v.is_travel() {
                option_color(v.move_type)
            } else {
                range.get_color_at(value)
            }
        };
        let layer_time = || {
            self.layers
                .get_layer_time(self.settings.time_mode, v.layer_id as usize)
        };
        match view_type {
            ViewType::FeatureType if v.is_travel() => option_color(v.move_type),
            ViewType::FeatureType => DEFAULT_EXTRUSION_ROLES_COLORS[v.role.index()],
            ViewType::Height => extrusion(&ranges.height, v.height),
            ViewType::Width => extrusion(&ranges.width, v.width),
            ViewType::Speed => ranges.speed.get_color_at(v.feedrate),
            ViewType::ActualSpeed => ranges.actual_speed.get_color_at(v.actual_feedrate),
            ViewType::FanSpeed => extrusion(&ranges.fan_speed, v.fan_speed),
            ViewType::Temperature => extrusion(&ranges.temperature, v.temperature),
            ViewType::PressureAdvance => extrusion(&ranges.pressure_advance, v.pressure_advance),
            ViewType::Acceleration => ranges.acceleration.get_color_at(v.acceleration),
            ViewType::Jerk => ranges.jerk.get_color_at(v.jerk),
            ViewType::VolumetricFlowRate => extrusion(&ranges.volumetric_rate, v.volumetric_rate()),
            ViewType::ActualVolumetricFlowRate => {
                extrusion(&ranges.actual_volumetric_rate, v.actual_volumetric_rate())
            }
            ViewType::LayerTimeLinear => extrusion(&ranges.layer_time[0], layer_time()),
            ViewType::LayerTimeLogarithmic => extrusion(&ranges.layer_time[1], layer_time()),
            ViewType::Tool => self
                .tool_colors
                .get(usize::from(v.extruder_id))
                .copied()
                .unwrap_or(DUMMY_COLOR),
            ViewType::Summary | ViewType::ColorPrint => {
                // An empty palette (no colors passed to the slice) has no color to pick.
                if self
                    .layers
                    .layer_contains_colorprint_options(v.layer_id as usize)
                    || self.color_print_colors.is_empty()
                {
                    DUMMY_COLOR
                } else {
                    self.color_print_colors[usize::from(v.color_id) % self.color_print_colors.len()]
                }
            }
        }
    }

    /// `get_color_range` (`ViewerImpl.cpp:1599-1621`); `None` for the view
    /// types without a range (upstream `DUMMY_COLOR_RANGE`).
    pub fn get_color_range(&self, view_type: ViewType) -> Option<&ColorRange> {
        let ranges = &self.ranges;
        Some(match view_type {
            ViewType::Height => &ranges.height,
            ViewType::Width => &ranges.width,
            ViewType::Speed => &ranges.speed,
            ViewType::ActualSpeed => &ranges.actual_speed,
            ViewType::FanSpeed => &ranges.fan_speed,
            ViewType::Temperature => &ranges.temperature,
            ViewType::PressureAdvance => &ranges.pressure_advance,
            ViewType::Acceleration => &ranges.acceleration,
            ViewType::Jerk => &ranges.jerk,
            ViewType::VolumetricFlowRate => &ranges.volumetric_rate,
            ViewType::ActualVolumetricFlowRate => &ranges.actual_volumetric_rate,
            ViewType::LayerTimeLinear => &ranges.layer_time[0],
            ViewType::LayerTimeLogarithmic => &ranges.layer_time[1],
            ViewType::Summary | ViewType::FeatureType | ViewType::ColorPrint | ViewType::Tool => {
                return None;
            }
        })
    }

    /// `update_color_ranges` (`ViewerImpl.cpp:1817-1878`), cached on the
    /// role and option visibility used last time.
    fn update_color_ranges(&mut self) {
        if self.settings_used_for_ranges.as_ref().is_some_and(|used| {
            used.extrusion_roles_visibility == self.settings.extrusion_roles_visibility
                && used.options_visibility == self.settings.options_visibility
        }) {
            return;
        }
        let ranges = &mut self.ranges;
        let settings = &self.settings;
        ranges.reset();
        let custom_visible =
            settings.extrusion_roles_visibility[GCodeExtrusionRole::Custom.index()];
        let travels = settings.options_visibility[OptionType::Travels.index()];
        let wipes = settings.options_visibility[OptionType::Wipes.index()];
        for v in &self.vertices {
            if v.is_extrusion() {
                ranges.update_extrusion(v, custom_visible);
            }
            if (v.is_travel() && travels) || (v.is_wipe() && wipes) || v.is_extrusion() {
                ranges.speed.update(v.feedrate);
                ranges.actual_speed.update(v.actual_feedrate);
                ranges.acceleration.update(v.acceleration);
                ranges.jerk.update(v.jerk);
            }
        }
        for range in &mut ranges.layer_time {
            for t in self.layers.get_times(settings.time_mode) {
                range.update(t);
            }
        }
        self.settings_used_for_ranges = Some(self.settings.clone());
    }
}
