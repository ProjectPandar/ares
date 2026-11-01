// Read-only diagnostic: real LibVGCodeWrapper conversion plus the extracted
// GCodeViewer::update_sequential_view_current actual speed block.
#pragma once
#include <cassert>
#include <cmath>
#include <cstring>
#include <limits>
#include "slic3r/GUI/LibVGCode/LibVGCodeWrapper.hpp"
#include "libvgcode/include/Viewer.hpp"
#include "libvgcode/include/ColorRange.hpp"
#include "libslic3r/LocalesUtils.hpp"

namespace actual_speed_profile_source {
using libvgcode::PathVertex;
using Slic3r::ColorRGBA;
using Slic3r::EMoveType;

// GCodeViewer.hpp:47-59 `SequentialView::ActualSpeedImguiWidget` data members.
struct Item
{
    float pos{ 0.0f };
    float speed{ 0.0f };
    bool internal{ false };
};
struct Widget
{
    std::pair<float, float> y_range = { 0.0f, 0.0f };
    std::vector<std::pair<float, ColorRGBA>> levels;
    std::vector<Item> data;
};

#include "actual-speed-profile/kernel.hpp"

// ColorRange::update is private to ViewerImpl; explicit instantiation may
// name it, which lets the oracle fill a real libvgcode::ColorRange.
using ColorRangeUpdate = void (libvgcode::ColorRange::*)(float);
ColorRangeUpdate color_range_update();
template <ColorRangeUpdate Update> struct ColorRangeAccess
{
    friend ColorRangeUpdate color_range_update() { return Update; }
};
template struct ColorRangeAccess<&libvgcode::ColorRange::update>;

// ViewerImpl.cpp:1858-1862, the m_actual_speed_range member only.
inline libvgcode::ColorRange actual_speed_range(const std::vector<PathVertex>& vertices, bool travels, bool wipes)
{
    libvgcode::ColorRange range;
    for (const PathVertex& v : vertices)
        if ((v.is_travel() && travels) || (v.is_wipe() && wipes) || v.is_extrusion())
            (range.*color_range_update())(v.actual_feedrate);
    return range;
}

inline uint32_t bits(float value)
{
    uint32_t result;
    std::memcpy(&result, &value, sizeof(result));
    return result;
}

inline nlohmann::json profiles(const std::vector<PathVertex>& vertices, bool travels, bool wipes)
{
    const libvgcode::ColorRange range = actual_speed_range(vertices, travels, wipes);
    // The trigger `enabled_range[1] != visible_last` is left to the consumer.
    const libvgcode::Interval enabled = { 0, std::numeric_limits<uint32_t>::max() };
    auto rows = nlohmann::json::array();
    for (size_t id = 0; id < vertices.size(); ++id) {
        Widget widget;
        widget.y_range = { std::nanf(""), std::nanf("") };
        update_actual_speed(vertices, enabled, id, range, widget);
        const bool updated = !std::isnan(widget.y_range.first);
        nlohmann::json row = { { "id", id }, { "updated", updated } };
        if (updated) {
            row["yRangeBits"] = { bits(widget.y_range.first), bits(widget.y_range.second) };
            auto levels = nlohmann::json::array();
            for (const auto& [value, color] : widget.levels)
                levels.push_back({ bits(value), bits(color.r()), bits(color.g()), bits(color.b()), bits(color.a()) });
            row["levels"] = levels;
            auto data = nlohmann::json::array();
            for (const Item& item : widget.data)
                data.push_back({ bits(item.pos), bits(item.speed), item.internal });
            row["data"] = data;
        }
        rows.push_back(std::move(row));
    }
    return rows;
}

inline bool dump(const Slic3r::GCodeProcessorResult& result, const char* path)
{
    Slic3r::CNumericLocalesSetter locale;
    const libvgcode::Viewer viewer; // conversion uses no GL context or viewer state
    const auto input = libvgcode::convert(result, result.extruder_colors, {}, viewer);
    auto vertices = nlohmann::json::array();
    for (size_t id = 0; id < input.vertices.size(); ++id) {
        const PathVertex& v = input.vertices[id];
        vertices.push_back({ { "id", id }, { "gcodeId", v.gcode_id }, { "type", static_cast<int>(v.type) },
            { "role", static_cast<int>(v.role) }, { "layerId", v.layer_id },
            { "positionBits", { bits(v.position[0]), bits(v.position[1]), bits(v.position[2]) } },
            { "timesBits", { bits(v.times[0]), bits(v.times[1]) } }, { "actualFeedrateBits", bits(v.actual_feedrate) } });
    }
    const nlohmann::json data = {
        { "oracle", "native source-kernel (not full Viewer execution)" },
        { "upstream", "345efec64c7b95a05ce5cf96df0dde66a158835c" },
        { "vertices", vertices },
        // Settings.hpp:34-35 hide travels and wipes by default.
        { "hidden", profiles(input.vertices, false, false) },
        { "shown", profiles(input.vertices, true, true) },
    };
    std::ofstream out(path);
    out << data.dump(2) << '\n';
    return bool(out);
}
}
