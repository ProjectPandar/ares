#ifndef slic3r_CoolingBuffer_shim_hpp_
#define slic3r_CoolingBuffer_shim_hpp_
#include <algorithm>
#include <cfloat>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <map>
#include <string>
#include <vector>

#ifndef EPSILON
#define EPSILON 1e-4
#endif
#ifndef SCALED_EPSILON
#define SCALED_EPSILON 1e-9
#endif

namespace Slic3r {

class GCode;
struct PerExtruderAdjustments;

// --- config shim ---------------------------------------------------------
template <typename T>
struct VecOpt {
    std::vector<T> values;
    T get_at(size_t i) const { return values.empty() ? T{} : values[std::min(i, values.size() - 1)]; }
};
struct PrintConfig {
    VecOpt<bool>   slow_down_for_layer_cooling{{true}};
    VecOpt<float>  slow_down_layer_time{{50.f}};
    VecOpt<float>  slow_down_min_speed{{1.f}};
    VecOpt<bool>   dont_slow_down_outer_wall{{false}};
    bool           use_relative_e_distances = true;
    VecOpt<float>  fan_min_speed{{0.f}};
    VecOpt<bool>   reduce_fan_stop_start_freq{{false}};
    VecOpt<int>    slow_down_layers{{0}};
    VecOpt<float>  fan_cooling_layer_time{{0.f}};
    VecOpt<bool>   fan_always_cooling{{false}};
    VecOpt<int>    disable_fan_first_layers{{0}};
    VecOpt<int>    close_fan_the_first_x_layers{{0}};
    VecOpt<int>    full_fan_speed_layer{{0}};
    VecOpt<int>    enable_overhang_bridge_fan{{0}};
    VecOpt<float>  overhang_fan_speed{{0.f}};
    VecOpt<float>  overhang_fan_threshold{{0.f}};
    VecOpt<int>    additional_cooling_fan_speed{{0}};
    bool           additional_cooling_fan{false};
    VecOpt<float>  slowdown_for_curled_perimeters{{0}};
    VecOpt<float>  travel_speed{{300.f}};
    VecOpt<int>    ironing_fan_speed{{0}};
    VecOpt<int>    internal_bridge_fan_speed{{0}};
    VecOpt<int>    support_material_interface_fan_speed{{0}};
    VecOpt<int>    fan_max_speed{{0}};
    VecOpt<bool>   cooling{{true}};
    VecOpt<float>  part_cooling_fan_min_pwm{{0}};
    VecOpt<bool>   auxiliary_fan{{false}};
    int gcode_flavor = 0;
};
struct Extruder { unsigned id_ = 0; unsigned id() const { return id_; } };
struct Vec3d { double v[3]{0,0,0}; double x() const { return v[0]; } double y() const { return v[1]; } double z() const { return v[2]; } };
struct GCode {
    PrintConfig cfg;
    const PrintConfig &config() const { return cfg; }
    struct Writer {
        Vec3d pos;
        Vec3d get_position() const { return pos; }
        std::string toolchange_prefix() const { return "; CHANGE FILAMENT"; }
        std::vector<Extruder> extruders_{{}};
        std::vector<Extruder> &extruders() { return extruders_; }
    } w;
    Writer &writer() { return w; }
};

// --- small string helpers ------------------------------------------------
inline bool shim_contains(const std::string &s, const std::string &n) { return s.find(n) != std::string::npos; }
inline bool shim_starts_with(const std::string &s, const char *p) { return s.rfind(p, 0) == 0; }
inline void shim_replace_all(std::string &s, const std::string &f, const std::string &t) {
    size_t pos = 0; while ((pos = s.find(f, pos)) != std::string::npos) { s.replace(pos, f.size(), t); pos += t.size(); }
}

struct Vec3f { float v[3]{0,0,0}; Vec3f() = default; Vec3f(float a, float b, float c) { v[0]=a; v[1]=b; v[2]=c; } float operator[](int i) const { return v[i]; } };
inline float shim_arc_length(const Vec3f &a, const Vec3f &b, const Vec3f &c, bool) {
    double dx = b[0]-a[0], dy = b[1]-a[1], d2 = b[2]-a[2];
    return float(std::sqrt(dx*dx+dy*dy+d2*d2));
}
inline bool is_decimal_separator_point() { return true; }
inline std::string shim_set_fan(int speed, unsigned int /*pwm*/) { char b[64]; std::snprintf(b, sizeof b, "M106 S%d\n", int(255.5f * std::max(0, std::min(100, speed)) / 100.f)); return b; }
inline std::string shim_set_additional_fan(int speed) { char b[64]; std::snprintf(b, sizeof b, "M106 P2 S%d\n", int(255.5f * std::max(0, std::min(100, speed)) / 100.f)); return b; }
inline bool is_approx(double a, double b) { return std::abs(a-b) < 1e-12; }

// --- vendored CoolingBuffer class ---------------------------------------
class CoolingBuffer {
public:
    CoolingBuffer(GCode &gcodegen);
    void        reset(const Vec3d &position);
    void        set_current_extruder(unsigned int extruder_id) { m_current_extruder = extruder_id; }
    std::string process_layer(std::string &&gcode, size_t layer_id, bool flush);

private:
	CoolingBuffer& operator=(const CoolingBuffer&) = delete;
    std::vector<PerExtruderAdjustments> parse_layer_gcode(const std::string &gcode, std::vector<float> &current_pos) const;
    float       calculate_layer_slowdown(std::vector<PerExtruderAdjustments> &per_extruder_adjustments);
    // Apply slow down over G-code lines stored in per_extruder_adjustments, enable fan if needed.
    // Returns the adjusted G-code.
    std::string apply_layer_cooldown(const std::string &gcode, size_t layer_id, float layer_time, std::vector<PerExtruderAdjustments> &per_extruder_adjustments);

    // G-code snippet cached for the support layers preceding an object layer.
    std::string                 m_gcode;
    // Internal data.
    // BBS: X,Y,Z,E,F,I,J
    std::vector<char>           m_axis;
    std::vector<float>          m_current_pos;
    // Current known fan speed or -1 if not known yet.
    int                         m_fan_speed;
    int                         m_additional_fan_speed;
    // Cached from GCodeWriter.
    // Printing extruder IDs, zero based.
    std::vector<unsigned int>   m_extruder_ids;
    // Highest of m_extruder_ids plus 1.
    unsigned int                m_num_extruders { 0 };
    const std::string           m_toolchange_prefix;
    // Referencs GCode::m_config, which is FullPrintConfig. While the PrintObjectConfig slice of FullPrintConfig is being modified,
    // the PrintConfig slice of FullPrintConfig is constant, thus no thread synchronization is required.
    const PrintConfig          &m_config;
    unsigned int                m_current_extruder;
    //BBS: current fan speed
    int                         m_current_fan_speed;
};


} // namespace Slic3r
#endif
