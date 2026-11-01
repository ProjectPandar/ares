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

// ViewerImpl.cpp:1189-1213 load-time caches, rebuilt locally over the dumped
// vertex array; convert() renumbers layer ids consecutively
// (LibVGCodeWrapper.cpp:260-266), so the layer count is the maximum id + 1.
inline std::pair<std::vector<uint32_t>, std::array<std::vector<float>, libvgcode::TIME_MODES_COUNT>>
layer_caches(const std::vector<PathVertex>& vertices)
{
    std::vector<uint32_t> layer_first_vertex;
    std::array<std::vector<float>, libvgcode::TIME_MODES_COUNT> layer_start_times;
    size_t layers_count = 0;
    for (const PathVertex& v : vertices)
        layers_count = std::max(layers_count, static_cast<size_t>(v.layer_id) + 1);
    const uint32_t vertices_count = static_cast<uint32_t>(vertices.size());
    layer_first_vertex.assign(layers_count, vertices_count);
    for (uint32_t i = vertices_count; i > 0; --i) {
        const uint32_t layer_id = vertices[i - 1].layer_id;
        if (layer_id < layer_first_vertex.size())
            layer_first_vertex[layer_id] = i - 1;
    }
    for (size_t i = layer_first_vertex.size(); i > 1; --i)
        layer_first_vertex[i - 2] = std::min(layer_first_vertex[i - 2], layer_first_vertex[i - 1]);
    std::array<float, libvgcode::TIME_MODES_COUNT> running{};
    for (std::vector<float>& times : layer_start_times)
        times.assign(layer_first_vertex.size(), 0.0f);
    size_t layer = 0;
    for (size_t i = 0; i <= vertices.size(); ++i) {
        for (; layer < layer_first_vertex.size() && layer_first_vertex[layer] == i; ++layer)
            for (size_t j = 0; j < libvgcode::TIME_MODES_COUNT; ++j)
                layer_start_times[j][layer] = running[j];
        if (i < vertices.size())
            for (size_t j = 0; j < libvgcode::TIME_MODES_COUNT; ++j)
                running[j] += vertices[i].times[j];
    }
    return { std::move(layer_first_vertex), std::move(layer_start_times) };
}

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
    const auto [layer_first_vertex, layer_start_times] = layer_caches(vertices);
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
            const float elapsed = get_estimated_time_at(vertices, layer_first_vertex, layer_start_times, id, static_cast<ETimeMode>(mode));
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
        {"upstream", "345efec64c7b95a05ce5cf96df0dde66a158835c"},
        {"vertices", records(input.vertices)}, {"synthetic", records(synthetic)}, {"formats", formats}
    };
    std::ofstream out(path);
    out << data.dump(2) << '\n';
    return bool(out);
}
}
