// ViewerImpl.cpp:35-36,49-55,59-75 rounding helpers and :315-349 default palettes;
// ViewerImpl.hpp members read by the color code; ViewerImpl.cpp:1856-1886 palette
// accessors, :2122-2183 update_color_ranges and :1734-1823 get_vertex_color as
// members of a GL-free holder of the same members (ViewerImpl::load needs GL).
namespace libvgcode {
template<class T, class O = T>
using IntegerOnly = std::enable_if_t<std::is_integral<T>::value, O>;
template<typename I>
inline IntegerOnly<I, I> fast_round_up(double a)
{
    // Why does Java Math.round(0.49999999999999994) return 1?
    // https://stackoverflow.com/questions/9902968/why-does-math-round0-49999999999999994-return-1
    return a == 0.49999999999999994 ? I(0) : I(floor(a + 0.5));
}
static float round_to_bin(const float value)
{
//    assert(value >= 0);
    constexpr float const scale[5]     = { 100.f,  1000.f,  10000.f,  100000.f,  1000000.f };
    constexpr float const invscale[5]  = { 0.01f,  0.001f,  0.0001f,  0.00001f,  0.000001f };
    constexpr float const threshold[5] = { 0.095f, 0.0095f, 0.00095f, 0.000095f, 0.0000095f };
    // Scaling factor, pointer to the tables above.
    int                   i = 0;
    // While the scaling factor is not yet large enough to get two integer digits after scaling and rounding:
    for (; value < threshold[i] && i < 4; ++i);
    // At least on MSVC std::round() calls a complex function, which is pretty expensive.
    // our fast_round_up is much cheaper and it could be inlined.
//    return std::round(value * scale[i]) * invscale[i];
    double a = value * scale[i];
    assert(std::abs(a) < double(std::numeric_limits<int64_t>::max()));
    return fast_round_up<int64_t>(a) * invscale[i];
}
static const std::array<Color, size_t(EGCodeExtrusionRole::COUNT)> DEFAULT_EXTRUSION_ROLES_COLORS = { {
    { 230, 179, 179 }, // None
    { 255, 230,  77 }, // Perimeter
    { 255, 125,  56 }, // ExternalPerimeter
    {  31,  31, 255 }, // OverhangPerimeter
    { 176,  48,  41 }, // InternalInfill
    { 150,  84, 204 }, // SolidInfill
    { 240,  64,  64 }, // TopSolidInfill
    { 255, 140, 105 }, // Ironing
    {  77, 128, 186 }, // BridgeInfill
    { 255, 255, 255 }, // GapFill
    {   0, 135, 110 }, // Skirt
    {   0, 255,   0 }, // SupportMaterial
    {   0, 128,   0 }, // SupportMaterialInterface
    { 179, 227, 171 }, // WipeTower
    {  94, 209, 148 },  // Custom
    // ORCA
    { 102,  92, 199 }, // BottomSurface
    {  77, 128, 186 }, // InternalBridgeInfill
    {   0,  59, 110 }, // Brim
    {   0,  64,   0 }, // SupportTransition
    { 128, 128, 128 }, // Mixed
} };
static const std::array<Color, size_t(EOptionType::COUNT)> DEFAULT_OPTIONS_COLORS{ {
    {  56,  72, 155 }, // Travels
    { 255, 255,   0 }, // Wipes
    { 205,  34, 214 }, // Retractions
    {  73, 173, 207 }, // Unretractions
    { 230, 230, 230 }, // Seams
    { 193, 190,  99 }, // ToolChanges
    { 218, 148, 139 }, // ColorChanges
    {  82, 240, 131 }, // PausePrints
    { 226, 210,  67 }  // CustomGCodes
} };

struct ColorsViewer
{
    Settings m_settings;
    Layers m_layers;
    std::array<Color, size_t(EGCodeExtrusionRole::COUNT)> m_extrusion_roles_colors;
    std::array<Color, size_t(EOptionType::COUNT)> m_options_colors;
    std::vector<PathVertex> m_vertices;
    std::optional<Settings> m_settings_used_for_ranges;
    ColorRange m_height_range;
    ColorRange m_width_range;
    ColorRange m_speed_range;
    ColorRange m_actual_speed_range;
    ColorRange m_fan_speed_range;
    ColorRange m_temperature_range;
    // ORCA: Add Pressure Advance visualization support
    ColorRange m_pressure_advance_range;
    // ORCA: Add Acceleration visualization support
    ColorRange m_acceleration_range;
    // ORCA: Add Jerk visualization support
    ColorRange m_jerk_range;
    ColorRange m_volumetric_rate_range;
    ColorRange m_actual_volumetric_rate_range;
    std::array<ColorRange, COLOR_RANGE_TYPES_COUNT> m_layer_time_range{
        ColorRange(EColorRangeType::Linear), ColorRange(EColorRangeType::Logarithmic)
    };
    PaletteWithAlpha m_tool_colors;
    PaletteWithAlpha m_color_print_colors;

const Color& get_extrusion_role_color(EGCodeExtrusionRole role) const
{
    return m_extrusion_roles_colors[size_t(role)];
}
void reset_default_extrusion_roles_colors()
{
    m_extrusion_roles_colors = DEFAULT_EXTRUSION_ROLES_COLORS;
}
const Color& get_option_color(EOptionType type) const
{
    return m_options_colors[size_t(type)];
}
void reset_default_options_colors()
{
    m_options_colors = DEFAULT_OPTIONS_COLORS;
}
void update_color_ranges()
{
    // Color ranges do not need to be recalculated that often. If the following settings are the same
    // as last time, the current ranges are still valid. The recalculation is quite expensive.
    if (m_settings_used_for_ranges.has_value() &&
        m_settings.extrusion_roles_visibility == m_settings_used_for_ranges->extrusion_roles_visibility &&
        m_settings.options_visibility == m_settings_used_for_ranges->options_visibility)
        return;

    m_width_range.reset();
    m_height_range.reset();
    m_speed_range.reset();
    m_actual_speed_range.reset();
    m_fan_speed_range.reset();
    m_temperature_range.reset();
    // ORCA: Add Pressure Advance visualization support
    m_pressure_advance_range.reset();
    // ORCA: Add Acceleration visualization support
    m_acceleration_range.reset();
    // ORCA: Add Jerk visualization support
    m_jerk_range.reset();
    m_volumetric_rate_range.reset();
    m_actual_volumetric_rate_range.reset();
    m_layer_time_range[0].reset(); // ColorRange::EType::Linear
    m_layer_time_range[1].reset(); // ColorRange::EType::Logarithmic

    for (size_t i = 0; i < m_vertices.size(); i++) {
        const PathVertex& v = m_vertices[i];
        if (v.is_extrusion()) {
            m_height_range.update(round_to_bin(v.height));
            if (!v.is_custom_gcode() || m_settings.extrusion_roles_visibility[size_t(EGCodeExtrusionRole::Custom)]) {
                m_width_range.update(round_to_bin(v.width));
                m_volumetric_rate_range.update(round_to_bin(v.volumetric_rate()));
                m_actual_volumetric_rate_range.update(round_to_bin(v.actual_volumetric_rate()));
            }
            m_fan_speed_range.update(round_to_bin(v.fan_speed));
            m_temperature_range.update(round_to_bin(v.temperature));
            // ORCA: Add Pressure Advance visualization support
            if (v.pressure_advance >= 0.0f)
                m_pressure_advance_range.update(v.pressure_advance);
        }
        if ((v.is_travel() && m_settings.options_visibility[size_t(EOptionType::Travels)]) ||
            (v.is_wipe() && m_settings.options_visibility[size_t(EOptionType::Wipes)]) ||
             v.is_extrusion()) {
            m_speed_range.update(v.feedrate);
            m_actual_speed_range.update(v.actual_feedrate);
            // ORCA: Add Acceleration visualization support
            m_acceleration_range.update(v.acceleration);
            // ORCA: Add Jerk visualization support
            m_jerk_range.update(v.jerk);
        }
    }

    const std::vector<float> times = m_layers.get_times(m_settings.time_mode);
    for (size_t i = 0; i < m_layer_time_range.size(); ++i) {
        for (float t : times) {
            m_layer_time_range[i].update(t);
        }
    }

    m_settings_used_for_ranges = m_settings;
}
Color get_vertex_color(const PathVertex& v) const
{
    if (v.type == EMoveType::Noop)
        return DUMMY_COLOR;

    if ((v.is_wipe() && (m_settings.view_type != EViewType::Speed && m_settings.view_type != EViewType::ActualSpeed && m_settings.view_type != EViewType::Acceleration && m_settings.view_type != EViewType::Jerk)) || v.is_option())
        return get_option_color(move_type_to_option(v.type));

    switch (m_settings.view_type)
    {
    case EViewType::FeatureType:
    {
        return v.is_travel() ? get_option_color(move_type_to_option(v.type)) : get_extrusion_role_color(v.role);
    }
    case EViewType::Height:
    {
        return v.is_travel() ? get_option_color(move_type_to_option(v.type)) : m_height_range.get_color_at(v.height);
    }
    case EViewType::Width:
    {
        return v.is_travel() ? get_option_color(move_type_to_option(v.type)) : m_width_range.get_color_at(v.width);
    }
    case EViewType::Speed:
    {
        return m_speed_range.get_color_at(v.feedrate);
    }
    case EViewType::ActualSpeed:
    {
        return m_actual_speed_range.get_color_at(v.actual_feedrate);
    }
    case EViewType::FanSpeed:
    {
        return v.is_travel() ? get_option_color(move_type_to_option(v.type)) : m_fan_speed_range.get_color_at(v.fan_speed);
    }
    case EViewType::Temperature:
    {
        return v.is_travel() ? get_option_color(move_type_to_option(v.type)) : m_temperature_range.get_color_at(v.temperature);
    }
// ORCA: Add Pressure Advance visualization support
    case EViewType::PressureAdvance:
    {
        return v.is_travel() ? get_option_color(move_type_to_option(v.type)) : m_pressure_advance_range.get_color_at(v.pressure_advance);
    }
    // ORCA: Add Acceleration visualization support
    case EViewType::Acceleration:
    {
        return m_acceleration_range.get_color_at(v.acceleration);
    }
    // ORCA: Add Jerk visualization support
    case EViewType::Jerk:
    {
        return m_jerk_range.get_color_at(v.jerk);
    }
    case EViewType::VolumetricFlowRate:
    {
        return v.is_travel() ? get_option_color(move_type_to_option(v.type)) : m_volumetric_rate_range.get_color_at(v.volumetric_rate());
    }
    case EViewType::ActualVolumetricFlowRate:
    {
        return v.is_travel() ? get_option_color(move_type_to_option(v.type)) : m_actual_volumetric_rate_range.get_color_at(v.actual_volumetric_rate());
    }
    case EViewType::LayerTimeLinear:
    {
        return v.is_travel() ? get_option_color(move_type_to_option(v.type)) :
            m_layer_time_range[0].get_color_at(m_layers.get_layer_time(m_settings.time_mode, static_cast<size_t>(v.layer_id)));
    }
    case EViewType::LayerTimeLogarithmic:
    {
        return v.is_travel() ? get_option_color(move_type_to_option(v.type)) :
            m_layer_time_range[1].get_color_at(m_layers.get_layer_time(m_settings.time_mode, static_cast<size_t>(v.layer_id)));
    }
    case EViewType::Tool:
    {
        assert(static_cast<size_t>(v.extruder_id) < m_tool_colors.size());
        const ColorWithAlpha& color = m_tool_colors[v.extruder_id];
        return { color[0], color[1], color[2] };
    }
    case EViewType::Summary: // ORCA
    case EViewType::ColorPrint:
    {
        if (m_layers.layer_contains_colorprint_options(static_cast<size_t>(v.layer_id)))
            return DUMMY_COLOR;
        const ColorWithAlpha& color = m_color_print_colors[static_cast<size_t>(v.color_id) % m_color_print_colors.size()];
        return { color[0], color[1], color[2] };
    }
    default: { break; }
    }

    return DUMMY_COLOR;
}
};
}
