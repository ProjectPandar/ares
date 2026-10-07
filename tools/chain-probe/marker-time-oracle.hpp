// Read-only diagnostic: real LibVGCodeWrapper conversion plus source kernel.
#pragma once
#include <numeric>
#include "slic3r/GUI/LibVGCode/LibVGCodeWrapper.hpp"
#include "libvgcode/include/Viewer.hpp"
#include "libslic3r/LocalesUtils.hpp"

namespace marker_time_source {
using libvgcode::PathVertex;
using libvgcode::ETimeMode;
#include "marker-time/kernel.hpp"

inline uint32_t bits(float value)
{
    uint32_t result;
    std::memcpy(&result, &value, sizeof(result));
    return result;
}

inline std::string text(float elapsed, float duration)
{
    // GCodeViewer.cpp:452-454; Utils.hpp function is used without modification.
    char buffer[128];
    std::snprintf(buffer, sizeof(buffer), "%s (%.3fs)",
        Slic3r::get_time_dhms(elapsed).c_str(), duration);
    return buffer;
}

inline nlohmann::json records(const std::vector<PathVertex>& vertices)
{
    auto rows = nlohmann::json::array();
    for (size_t id = 0; id < vertices.size(); ++id) {
        const auto& vertex = vertices[id];
        nlohmann::json row = {
            {"id", id}, {"gcodeId", vertex.gcode_id}, {"layerId", vertex.layer_id},
            {"type", static_cast<int>(vertex.type)}, {"role", static_cast<int>(vertex.role)},
            {"extruderId", vertex.extruder_id}, {"colorId", vertex.color_id}, {"position", vertex.position},
            {"durationBits", nlohmann::json::array()},
            {"elapsedBits", nlohmann::json::array()}, {"text", nlohmann::json::array()}
        };
        for (size_t mode = 0; mode < libvgcode::TIME_MODES_COUNT; ++mode) {
            const float elapsed = get_estimated_time_at(vertices, id, static_cast<ETimeMode>(mode));
            row["durationBits"].push_back(bits(vertex.times[mode]));
            row["elapsedBits"].push_back(bits(elapsed));
            row["text"].push_back(text(elapsed, vertex.times[mode]));
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
    std::vector<PathVertex> synthetic(6);
    const std::array<std::array<float, 2>, 6> durations = {{
        {0.0f, 0.0f}, {0.1f, 0.2f}, {0.2f, 0.1f},
        {16777216.0f, 0.3f}, {1.0f, 0.00001f}, {1.0f, 0.00001f}
    }};
    for (size_t i = 0; i < synthetic.size(); ++i) {
        synthetic[i].position = { static_cast<float>(i), 0.0f, 0.0f };
        synthetic[i].gcode_id = static_cast<uint32_t>(i + 1);
        synthetic[i].type = libvgcode::EMoveType::Extrude;
        synthetic[i].times = durations[i];
    }
    auto formats = nlohmann::json::array();
    for (float elapsed : {0.0f, std::nextafter(1.0f, 0.0f), 1.0f, std::nextafter(1.0f, 2.0f),
            std::nextafter(60.0f, 0.0f), 60.0f, std::nextafter(60.0f, 61.0f),
            3599.999f, 3600.0f, 86399.99f, 86400.0f, 90061.25f}) {
        for (float duration : {0.0f, 0.0005f, 0.0625f, 1.2345f})
            formats.push_back({{"elapsedBits", bits(elapsed)}, {"durationBits", bits(duration)},
                {"text", text(elapsed, duration)}});
    }
    const nlohmann::json data = {
        {"oracle", "native source-kernel (not full Viewer execution)"},
        {"upstream", "8500fcdccaa10b5099ac20d252af3a7c560046f1"},
        {"vertices", records(input.vertices)}, {"synthetic", records(synthetic)}, {"formats", formats}
    };
    std::ofstream out(path);
    out << data.dump(2) << '\n';
    return bool(out);
}
}
