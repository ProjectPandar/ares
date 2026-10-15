// ViewerImpl.hpp:234,238 radius defaults, ViewerImpl.cpp:909-983 Vec4 and extract_pos_and_or_hwa,
// ViewerImpl.cpp:1648-1658 radius setters over a caller-owned radius.
namespace libvgcode {
    inline constexpr float default_travels_radius{ DEFAULT_TRAVELS_RADIUS_MM };

    inline constexpr float default_wipes_radius{ DEFAULT_WIPES_RADIUS_MM };

using Vec4 = std::array<float, 4>;

inline void extract_pos_and_or_hwa(const std::vector<PathVertex>& vertices, float travels_radius, float wipes_radius, BitSet<>& valid_lines_bitset,
    std::vector<Vec4>* positions = nullptr, std::vector<Vec4>* heights_widths_angles = nullptr, bool update_bitset = false) {
  static constexpr const Vec3 ZERO = { 0.0f, 0.0f, 0.0f };
    if (positions == nullptr && heights_widths_angles == nullptr)
        return;
    if (vertices.empty())
        return;
    if (travels_radius <= 0.0f || wipes_radius <= 0.0f)
        return;

    if (positions != nullptr)
        positions->reserve(vertices.size());
    if (heights_widths_angles != nullptr)
        heights_widths_angles->reserve(vertices.size());
    for (size_t i = 0; i < vertices.size(); ++i) {
        const PathVertex& v = vertices[i];
        const EMoveType move_type = v.type;
        const bool prev_line_valid = i > 0 && valid_lines_bitset[i - 1];
        const Vec3 prev_line = prev_line_valid ? v.position - vertices[i - 1].position : ZERO;
        const bool this_line_valid = i + 1 < vertices.size() &&
                                     vertices[i + 1].position != v.position &&
                                     vertices[i + 1].type == move_type &&
                                     move_type != EMoveType::Seam;
        const Vec3 this_line = this_line_valid ? vertices[i + 1].position - v.position : ZERO;

        if (this_line_valid) {
            // there is a valid path between point i and i+1.
        }
        else {
            // the connection is invalid, there should be no line rendered, ever
            if (update_bitset)
                valid_lines_bitset.reset(i);
        }
        
        if (positions != nullptr) {
            // the last component is a dummy float to comply with GL_RGBA32F format
            Vec4 position = { v.position[0], v.position[1], v.position[2], 0.0f };
            if (move_type == EMoveType::Extrude)
                // push down extrusion vertices by half height to render them at the right z
                position[2] -= 0.5f * v.height;
            positions->emplace_back(position);
        }

        if (heights_widths_angles != nullptr) {
            float height = 0.0f;
            float width = 0.0f;
            if (v.is_travel()) {
                height = travels_radius;
                width  = travels_radius;
            }
            else if (v.is_wipe()) {
                height = wipes_radius;
                width  = wipes_radius;
            }
            else {
                height = v.height;
                width = v.width;
            }
            
            // ORCA: Set bias for wipes and options to avoid z-fighting
            float bias = 0.0f;
            if (v.is_wipe())
                bias = 0.05f;
            else if (v.is_option())
                bias = 0.1f;

            // the last component is a dummy float to comply with GL_RGBA32F format
            // ORCA: Pass bias to shader
            heights_widths_angles->push_back({ height, width,
                std::atan2(prev_line[0] * this_line[1] - prev_line[1] * this_line[0], dot(prev_line, this_line)), bias });
        }
    }
}

inline void set_travels_radius(float& m_travels_radius, float radius)
{
    m_travels_radius = std::clamp(radius, MIN_TRAVELS_RADIUS_MM, MAX_TRAVELS_RADIUS_MM);
}

inline void set_wipes_radius(float& m_wipes_radius, float radius)
{
    m_wipes_radius = std::clamp(radius, MIN_WIPES_RADIUS_MM, MAX_WIPES_RADIUS_MM);
}
}
