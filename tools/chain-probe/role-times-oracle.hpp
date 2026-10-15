// Read-only diagnostic: real LibVGCodeWrapper conversion, the real libvgcode
// ExtrusionRoles, the extracted ViewerImpl::load time/role accumulators
// (ViewerImpl.cpp:1002-1018) and the extracted GCodeViewer::render_legend
// Feature Type Time/% cells (GCodeViewer.cpp:3151, 3410-3418, 3638-3668).
#pragma once
#include <array>
#include <cassert>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>
#include "slic3r/GUI/LibVGCode/LibVGCodeWrapper.hpp"
#include "libvgcode/include/Viewer.hpp"
#include "libvgcode/src/ExtrusionRoles.hpp"
#include "libslic3r/ExtrusionEntity.hpp"
#include "libslic3r/LocalesUtils.hpp"
#include "libslic3r/Utils.hpp"

namespace role_times_source {
using Slic3r::ExtrusionEntity;
using Slic3r::get_time_dhms;
using Slic3r::short_time;

// English translation is the identity; the GUI macro (I18N.hpp:7) needs a wx app.
#pragma push_macro("_u8L")
#undef _u8L
inline std::string _u8L(const std::string& text) { return text; }

// The libvgcode::Viewer queries the cells read (ViewerImpl.hpp:136, 141, 142, 147)
// over the accumulators ViewerImpl::load fills, in the default Normal time mode.
struct RolesViewer
{
    std::array<float, libvgcode::TIME_MODES_COUNT> total_time{ 0.0f, 0.0f };
    std::array<float, libvgcode::TIME_MODES_COUNT> travels_time{ 0.0f, 0.0f };
    libvgcode::ExtrusionRoles extrusion_roles;
    libvgcode::ETimeMode time_mode{ libvgcode::ETimeMode::Normal };
    float get_estimated_time() const { return total_time[static_cast<size_t>(time_mode)]; }
    std::vector<libvgcode::EGCodeExtrusionRole> get_extrusion_roles() const { return extrusion_roles.get_roles(); }
    float get_extrusion_role_estimated_time(libvgcode::EGCodeExtrusionRole role) const { return extrusion_roles.get_time(role, time_mode); }
    float get_travels_estimated_time() const { return travels_time[static_cast<size_t>(time_mode)]; }
};

#include "role-times/kernel.hpp"
#pragma pop_macro("_u8L")

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
    const auto input = libvgcode::convert(result, result.extruder_colors, {}, viewer);
    RolesViewer roles_viewer;
    load_times(input.vertices, roles_viewer.total_time, roles_viewer.travels_time, roles_viewer.extrusion_roles);

    auto vertices = nlohmann::json::array();
    for (size_t id = 0; id < input.vertices.size(); ++id) {
        const libvgcode::PathVertex& v = input.vertices[id];
        vertices.push_back({ { "id", id }, { "type", static_cast<int>(v.type) }, { "role", static_cast<int>(v.role) },
            { "timesBits", { bits(v.times[0]), bits(v.times[1]) } } });
    }
    auto roles = nlohmann::json::array();
    for (libvgcode::EGCodeExtrusionRole role : roles_viewer.get_extrusion_roles())
        roles.push_back({ { "role", static_cast<int>(role) },
            { "timesBits", { bits(roles_viewer.extrusion_roles.get_time(role, libvgcode::ETimeMode::Normal)),
                               bits(roles_viewer.extrusion_roles.get_time(role, libvgcode::ETimeMode::Stealth)) } } });

    const auto& mode = result.print_statistics.modes[static_cast<size_t>(Slic3r::PrintEstimatedStatistics::ETimeMode::Normal)];
    std::vector<std::string> labels, times, percents;
    std::string travel_time, travel_percent;
    feature_type_cells(roles_viewer, mode, labels, times, percents, travel_time, travel_percent);
    auto rows = nlohmann::json::array();
    for (size_t i = 0; i < labels.size(); ++i)
        rows.push_back({ { "label", labels[i] }, { "time", times[i] }, { "percent", percents[i] } });

    const nlohmann::json data = {
        { "oracle", "native source-kernel (not full Viewer execution)" },
        { "upstream", "8500fcdccaa10b5099ac20d252af3a7c560046f1" },
        { "vertices", vertices },
        { "roles", roles },
        { "travelsTimeBits", { bits(roles_viewer.travels_time[0]), bits(roles_viewer.travels_time[1]) } },
        { "totalTimeBits", { bits(roles_viewer.total_time[0]), bits(roles_viewer.total_time[1]) } },
        { "printStatisticsTimeBits", bits(mode.time) },
        { "cells", { { "rows", rows }, { "travelTime", travel_time }, { "travelPercent", travel_percent } } },
    };
    std::ofstream out(path);
    out << data.dump(2) << '\n';
    return bool(out);
}
}
