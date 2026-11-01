// Read-only diagnostic: real LibVGCodeWrapper conversion, the real libvgcode
// ColorRange and Layers, and the extracted ViewerImpl color code
// (ViewerImpl.cpp:1734-1823, 2122-2183) with the default Settings and palettes
// (ViewerImpl.cpp:770-771), per vertex in the Feature Type, Height, Speed, Width,
// Volumetric Flow Rate, Fan Speed and Temperature views, plus Speed with Travels
// visible (the speed range takes travel feedrates only then, :2163-2165).
#pragma once
#include <array>
#include <cassert>
#include <cmath>
#include <cstring>
#include <fstream>
#include <limits>
#include <optional>
#include <vector>
#include "slic3r/GUI/LibVGCode/LibVGCodeWrapper.hpp"
#include "libvgcode/include/Viewer.hpp"
#include "libvgcode/include/PathVertex.hpp"
#include "libvgcode/include/ColorRange.hpp"
#include "libvgcode/src/Layers.hpp"
#include "libvgcode/src/Settings.hpp"
#include "libvgcode/src/Utils.hpp"
#include "libslic3r/LocalesUtils.hpp"

#include "segment-colors/kernel.hpp"

namespace segment_colors_source {
inline uint32_t bits(float value)
{
    uint32_t result;
    std::memcpy(&result, &value, sizeof(result));
    return result;
}

inline bool dump(const Slic3r::GCodeProcessorResult& result, const char* path)
{
    Slic3r::CNumericLocalesSetter locale;
    const libvgcode::Viewer viewer; // conversion uses no GL context or viewer state
    auto input = libvgcode::convert(result, result.extruder_colors, {}, viewer);

    // ViewerImpl::ViewerImpl (:770-771) and the vertex move of ViewerImpl::load (:1130).
    libvgcode::ColorsViewer colors;
    colors.reset_default_extrusion_roles_colors();
    colors.reset_default_options_colors();
    colors.m_vertices = std::move(input.vertices);

    auto view_colors = [&colors](libvgcode::EViewType type) {
        colors.m_settings.view_type = type;
        colors.update_color_ranges();
        auto rgb = nlohmann::json::array();
        for (const libvgcode::PathVertex& v : colors.m_vertices) {
            const libvgcode::Color c = colors.get_vertex_color(v);
            rgb.push_back({ c[0], c[1], c[2] });
        }
        return rgb;
    };
    using libvgcode::EViewType;
    const std::array<std::pair<const char*, EViewType>, 7> views{ {
        { "featureType", EViewType::FeatureType }, { "height", EViewType::Height },
        { "speed", EViewType::Speed }, { "width", EViewType::Width },
        { "volumetricFlowRate", EViewType::VolumetricFlowRate },
        { "fanSpeed", EViewType::FanSpeed }, { "temperature", EViewType::Temperature },
    } };
    std::vector<nlohmann::json> view_rgb;
    for (const auto& [name, type] : views)
        view_rgb.push_back(view_colors(type));
    colors.m_settings.options_visibility[size_t(libvgcode::EOptionType::Travels)] = true;
    const auto speed_travels = view_colors(EViewType::Speed);

    auto dumped = nlohmann::json::array();
    for (size_t id = 0; id < colors.m_vertices.size(); ++id) {
        const libvgcode::PathVertex& v = colors.m_vertices[id];
        nlohmann::json row = { { "id", id }, { "type", static_cast<int>(v.type) },
            { "positionBits", { bits(v.position[0]), bits(v.position[1]), bits(v.position[2]) } } };
        for (size_t view = 0; view < views.size(); ++view)
            row[views[view].first] = view_rgb[view][id];
        row["speedTravels"] = speed_travels[id];
        dumped.push_back(std::move(row));
    }
    const nlohmann::json data = {
        { "oracle", "native source-kernel (not full Viewer execution)" },
        { "upstream", "345efec64c7b95a05ce5cf96df0dde66a158835c" },
        { "vertices", dumped },
    };
    std::ofstream out(path);
    out << data.dump(2) << '\n';
    return bool(out);
}
}
