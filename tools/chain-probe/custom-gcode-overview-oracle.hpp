// Read-only diagnostic: real LibVGCodeWrapper conversion, the real
// libvgcode::Layers fill of ViewerImpl::load (ViewerImpl.cpp:1002-1005) and the
// extracted GCodeViewer::render_legend Custom G-code overview (GCodeViewer.cpp:4477-4546).
#pragma once
#include <algorithm>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <iterator>
#include <string>
#include <vector>
#include "slic3r/GUI/LibVGCode/LibVGCodeWrapper.hpp"
#include "libvgcode/include/Viewer.hpp"
#include "libvgcode/src/Layers.hpp"
#include "libslic3r/CustomGCode.hpp"
#include "libslic3r/LocalesUtils.hpp"
#include "libslic3r/Utils.hpp"

namespace custom_gcode_overview_source {
using namespace Slic3r::CustomGCode;
namespace CustomGCode = Slic3r::CustomGCode;
using Slic3r::get_time_dhms;
using Slic3r::short_time;

// English translation is the identity; the GUI macro (I18N.hpp:7) needs a wx app.
#pragma push_macro("_u8L")
#undef _u8L
inline std::string _u8L(const char* text) { return text; }

// Layout calls only; the overview's data are the imgui.text cells.
struct ImVec2
{
    float x{ 0.0f };
    float y{ 0.0f };
};
namespace ImGui {
inline void Dummy(ImVec2) {}
inline void SameLine(float = 0.0f) {}
}

// The two libvgcode::Viewer queries the overview reads (Viewer.cpp:235-238, 335-338)
// over a Layers filled as ViewerImpl::load does, in the default Normal time mode.
struct LayersViewer
{
    libvgcode::Layers layers;
    std::vector<float> get_layers_zs() const { return layers.get_zs(); }
    std::vector<float> get_layers_estimated_times() const { return layers.get_times(libvgcode::ETimeMode::Normal); }
};

struct Cells
{
    std::vector<std::string> text;
    std::vector<float> times;
};

struct Imgui
{
    Cells& cells;
    void text(const std::string& value) { cells.text.push_back(value); }
    void time(float value) { cells.times.push_back(value); }
};

// The observer exercises the ordinary non-belt viewer context.
inline const std::vector<double> m_belt_layer_zs;
#include "custom-gcode-overview/kernel.hpp"
#pragma pop_macro("_u8L")

inline uint32_t bits(float value)
{
    uint32_t result;
    std::memcpy(&result, &value, sizeof(result));
    return result;
}

/// `items` is a plate-custom-gcodes.nix dump; `plate` the zero-based sliced plate key.
inline bool dump(const Slic3r::GCodeProcessorResult& result, const char* items, int plate, const char* path)
{
    Slic3r::CNumericLocalesSetter locale;
    std::vector<Item> custom_gcode_per_print_z;
    std::ifstream in(items);
    const nlohmann::json loaded = nlohmann::json::parse(in);
    for (const nlohmann::json& entry : loaded.at("plates"))
        if (entry.at("plate").get<int>() == plate)
            for (const nlohmann::json& code : entry.at("gcodes")) {
                Item item;
                item.print_z = code.at("print_z").get<double>();
                item.type = static_cast<Type>(code.at("type").get<int>());
                item.extruder = code.at("extruder").get<int>();
                item.color = code.at("color").get<std::string>();
                item.extra = code.at("extra").get<std::string>();
                custom_gcode_per_print_z.push_back(item);
            }

    const libvgcode::Viewer viewer; // conversion uses no GL context or viewer state
    const auto input = libvgcode::convert(result, result.extruder_colors, {}, viewer);
    LayersViewer layers_viewer;
    for (size_t i = 0; i < input.vertices.size(); ++i)
        layers_viewer.layers.update(input.vertices[i], static_cast<uint32_t>(i));

    Cells cells;
    Imgui imgui{ cells };
    render_custom_gcode_overview(custom_gcode_per_print_z, layers_viewer, imgui, 0.0f, 0.0f);

    auto zs = nlohmann::json::array();
    for (float z : layers_viewer.get_layers_zs())
        zs.push_back(bits(z));
    auto times = nlohmann::json::array();
    for (float time : layers_viewer.get_layers_estimated_times())
        times.push_back(bits(time));
    auto rows = nlohmann::json::array();
    for (size_t i = 0; i < custom_gcode_per_print_z.size(); ++i)
        rows.push_back({ { "printZ", custom_gcode_per_print_z[i].print_z },
            { "label", cells.text[3 * i] }, { "layer", cells.text[3 * i + 1] },
            { "timeBits", bits(cells.times[i]) }, { "time", cells.text[3 * i + 2] } });
    const nlohmann::json data = {
        { "oracle", "native source-kernel (not full Viewer execution)" },
        { "upstream", "345efec64c7b95a05ce5cf96df0dde66a158835c" },
        { "zsBits", zs },
        { "timesBits", times },
        { "rows", rows },
    };
    std::ofstream out(path);
    out << data.dump(2) << '\n';
    return bool(out);
}
}
