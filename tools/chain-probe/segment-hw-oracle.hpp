// Read-only diagnostic: real LibVGCodeWrapper conversion and the extracted
// libvgcode segment heights/widths (ViewerImpl.cpp:909-983, 1648-1658) with the
// ViewerImpl.hpp:234,238 default radii, called as ViewerImpl::load does
// (ViewerImpl.cpp:1058-1060, 1071).
#pragma once
#include <algorithm>
#include <array>
#include <cmath>
#include <cstring>
#include <fstream>
#include <vector>
#include "slic3r/GUI/LibVGCode/LibVGCodeWrapper.hpp"
#include "libvgcode/include/Viewer.hpp"
#include "libvgcode/include/PathVertex.hpp"
#include "libvgcode/src/Bitset.hpp"
#include "libvgcode/src/Utils.hpp"
#include "libslic3r/LocalesUtils.hpp"

#include "segment-hw/kernel.hpp"

namespace segment_hw_source {
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
    const std::vector<libvgcode::PathVertex>& vertices = input.vertices;

    libvgcode::BitSet<> valid_lines_bitset(vertices.size());
    valid_lines_bitset.setAll();
    std::vector<libvgcode::Vec4> positions;
    std::vector<libvgcode::Vec4> heights_widths_angles;
    libvgcode::extract_pos_and_or_hwa(vertices, libvgcode::default_travels_radius, libvgcode::default_wipes_radius,
        valid_lines_bitset, &positions, &heights_widths_angles, true);

    auto dumped = nlohmann::json::array();
    for (size_t id = 0; id < vertices.size(); ++id) {
        const libvgcode::PathVertex& v = vertices[id];
        const libvgcode::Vec4& hwa = heights_widths_angles[id];
        const libvgcode::Vec4& position = positions[id];
        dumped.push_back({ { "id", id }, { "type", static_cast<int>(v.type) }, { "role", static_cast<int>(v.role) },
            { "valid", static_cast<bool>(valid_lines_bitset[id]) },
            { "positionBits", { bits(v.position[0]), bits(v.position[1]), bits(v.position[2]) } },
            { "positionOutBits", { bits(position[0]), bits(position[1]), bits(position[2]) } },
            { "heightWidthBits", { bits(v.height), bits(v.width) } },
            { "hwaBits", { bits(hwa[0]), bits(hwa[1]), bits(hwa[2]), bits(hwa[3]) } } });
    }
    auto clamps = nlohmann::json::array();
    for (const float radius : { 0.01f, 0.05f, 0.1f, 0.3f, 1.0f, 2.0f }) {
        float travels = libvgcode::default_travels_radius;
        float wipes = libvgcode::default_wipes_radius;
        libvgcode::set_travels_radius(travels, radius);
        libvgcode::set_wipes_radius(wipes, radius);
        clamps.push_back({ bits(radius), bits(travels), bits(wipes) });
    }
    const nlohmann::json data = {
        { "oracle", "native source-kernel (not full Viewer execution)" },
        { "upstream", "8500fcdccaa10b5099ac20d252af3a7c560046f1" },
        { "defaultRadiiBits", { bits(libvgcode::default_travels_radius), bits(libvgcode::default_wipes_radius) } },
        { "radiusClampBits", clamps },
        { "vertices", dumped },
    };
    std::ofstream out(path);
    out << data.dump(2) << '\n';
    return bool(out);
}
}
