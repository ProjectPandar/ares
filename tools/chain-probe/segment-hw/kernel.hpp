// ViewerImpl.hpp:292,296 radius defaults, include/Types.hpp:42 Vec4,
// ViewerImpl.cpp:1030-1119 is_line/is_joint and extract_pos_and_or_hwa,
// ViewerImpl.cpp:1937-1947 radius setters over a caller-owned radius.
namespace libvgcode {
    inline constexpr float default_travels_radius{ DEFAULT_TRAVELS_RADIUS_MM };

    inline constexpr float default_wipes_radius{ DEFAULT_WIPES_RADIUS_MM };

using Vec4 = std::array<float, 4>;

inline bool is_line(const std::vector<PathVertex>& vertices, size_t i)
{
    return i + 1 < vertices.size() && vertices[i + 1].position != vertices[i].position &&
           vertices[i + 1].type == vertices[i].type && vertices[i].type != EMoveType::Seam;
}

// ORCA: whether two lines meet at vertex i, also across the vertex repeated in place that starts a path split
// where a value the preview colors by changes, see starts_path() in LibVGCodeWrapper.cpp
inline bool is_joint(const std::vector<PathVertex>& vertices, size_t i)
{
    auto repeated = [&vertices](size_t j) {
        return j + 1 < vertices.size() && vertices[j + 1].position == vertices[j].position && vertices[j + 1].type == vertices[j].type;
    };
    const bool before = i > 0 && (is_line(vertices, i - 1) || (i > 1 && repeated(i - 1) && is_line(vertices, i - 2)));
    const bool after = is_line(vertices, i) || (repeated(i) && is_line(vertices, i + 1));
    return before && after;
}

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
        const bool this_line_valid = is_line(vertices, i);
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
            // ORCA: the last component, there to comply with GL_RGBA32F format, carries the layer the shaders grey and dim by,
            // plus 0.5 at the joint of two segments, which the shaders read as an int, see joint_a in Segments_Vertex_Shader
            Vec4 position = { v.position[0], v.position[1], v.position[2],
                              static_cast<float>(v.layer_id) + (is_joint(vertices, i) ? 0.5f : 0.0f) };
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
