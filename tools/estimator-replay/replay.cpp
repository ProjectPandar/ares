// Standalone replay of the OrcaSlicer 2.4.2 GCodeProcessor time estimator.
//
// Embeds the upstream TimeMachine/Planner logic verbatim (marked with the
// upstream file:line each block was copied from) so per-block ground truth
// can be diffed against Ares `ARES_DUMP_BLOCKS` output. Diagnostic tool
// only — not part of the Rust build.
//
// Usage:
//   replay <input.gcode> <limits-file>
// limits-file: `key value` lines (mm/s or mm/s^2 units as in project_settings):
//   max_speed_x max_speed_y max_speed_z max_speed_e
//   max_accel_x max_accel_y max_accel_z max_accel_e
//   max_jerk_x max_jerk_y max_jerk_z max_jerk_e
//   accel travel_accel retract_accel  (machine_max_acceleration_* / *_traveling)
//   min_extruding min_travel
//
// Output (stdout): per processed block
//   <g1_line_id> <distance> <cruise> <accel> <entry> <exit> <safe> <time> <cumulative>
// and a final `TOTAL <seconds>` line.

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iostream>
#include <optional>
#include <sstream>
#include <string>
#include <vector>

enum Axis { X = 0, Y = 1, Z = 2, E = 3 };
using AxisCoords = std::array<float, 4>;

enum class EMoveType { Noop, Retract, Unretract, Extrude, Travel, Wipe };

static inline float sqr(float v) { return v * v; }

// ---- GCodeProcessor.cpp:131-155 ----
static float speed_from_distance(float initial_feedrate, float distance, float acceleration)
{
    const float value = std::max(0.0f, sqr(initial_feedrate) + 2.0f * acceleration * distance);
    return std::sqrt(value);
}
static float max_allowable_speed(float acceleration, float target_velocity, float distance)
{
    const float value = std::max(0.0f, sqr(target_velocity) - 2.0f * acceleration * distance);
    return std::sqrt(value);
}
static float estimated_acceleration_distance(float initial_rate, float target_rate, float acceleration)
{
    return (acceleration == 0.0f) ? 0.0f : (sqr(target_rate) - sqr(initial_rate)) / (2.0f * acceleration);
}
static float intersection_distance(float initial_rate, float final_rate, float acceleration, float distance)
{
    return (acceleration == 0.0f) ? 0.0f : (2.0f * acceleration * distance - sqr(initial_rate) + sqr(final_rate)) / (4.0f * acceleration);
}

// ---- GCodeProcessor.hpp:425-476 (TimeBlock) ----
struct TimeBlock {
    EMoveType move_type{ EMoveType::Noop };
    unsigned int g1_line_id{ 0 };
    float distance{ 0.0f };
    float acceleration{ 0.0f };
    float max_entry_speed{ 0.0f };
    float safe_feedrate{ 0.0f };
    struct Flags {
        bool recalculate{ false };
        bool nominal_length{ false };
        bool prepare_stage{ false };
    } flags;
    struct FeedrateProfile {
        float entry{ 0.0f };
        float cruise{ 0.0f };
        float exit{ 0.0f };
    } feedrate_profile;
    struct Trapezoid {
        float accelerate_until{ 0.0f };
        float decelerate_after{ 0.0f };
        float cruise_feedrate{ 0.0f };
        float acceleration_time(float entry_feedrate, float acceleration) const {
            // GCodeProcessor.cpp:200-206
            if (acceleration != 0.0f)
                return (speed_from_distance(entry_feedrate, accelerate_until, acceleration) - entry_feedrate) / acceleration;
            return 0.0f;
        }
        float cruise_time() const { return (cruise_feedrate != 0.0f) ? cruise_distance() / cruise_feedrate : 0.0f; }
        float deceleration_time(float distance, float acceleration) const {
            // GCodeProcessor.cpp:208-215
            if (acceleration != 0.0f)
                return (speed_from_distance(cruise_feedrate, distance - decelerate_after, -acceleration) - cruise_feedrate) / -acceleration;
            return 0.0f;
        }
        float cruise_distance() const { return decelerate_after - accelerate_until; }
    } trapezoid;

    // GCodeProcessor.cpp:255-274
    void calculate_trapezoid()
    {
        float accelerate_distance = std::max(0.0f, estimated_acceleration_distance(feedrate_profile.entry, feedrate_profile.cruise, acceleration));
        const float decelerate_distance = std::max(0.0f, estimated_acceleration_distance(feedrate_profile.cruise, feedrate_profile.exit, -acceleration));
        float cruise_distance = distance - accelerate_distance - decelerate_distance;
        if (cruise_distance < 0.0f) {
            accelerate_distance = std::clamp(intersection_distance(feedrate_profile.entry, feedrate_profile.exit, acceleration, distance), 0.0f, distance);
            cruise_distance = 0.0f;
            trapezoid.cruise_feedrate = speed_from_distance(feedrate_profile.entry, accelerate_distance, acceleration);
        } else
            trapezoid.cruise_feedrate = feedrate_profile.cruise;
        trapezoid.accelerate_until = accelerate_distance;
        trapezoid.decelerate_after = accelerate_distance + cruise_distance;
    }
    float time() const {
        // GCodeProcessor.hpp:473-476
        return trapezoid.acceleration_time(feedrate_profile.entry, acceleration) + trapezoid.cruise_time() + trapezoid.deceleration_time(distance, acceleration);
    }
};

// ---- GCodeProcessor.cpp:316-365 (planner kernels) ----
static void planner_forward_pass_kernel(const TimeBlock& prev, TimeBlock& curr)
{
    if (!prev.flags.nominal_length && prev.feedrate_profile.entry < curr.feedrate_profile.entry) {
        const float new_entry_speed = max_allowable_speed(-prev.acceleration, prev.feedrate_profile.entry, prev.distance);
        if (new_entry_speed < curr.feedrate_profile.entry) {
            curr.feedrate_profile.entry = new_entry_speed;
            curr.flags.recalculate = true;
        }
    }
}
static void planner_reverse_pass_kernel(TimeBlock& curr, const TimeBlock& next)
{
    const float max_entry_speed = curr.max_entry_speed;
    if (curr.feedrate_profile.entry != max_entry_speed || next.flags.recalculate) {
        const float new_entry_speed = curr.flags.nominal_length ? max_entry_speed :
            std::min(max_entry_speed, max_allowable_speed(-curr.acceleration, next.feedrate_profile.entry, curr.distance));
        if (curr.feedrate_profile.entry != new_entry_speed) {
            curr.feedrate_profile.entry = new_entry_speed;
            curr.flags.recalculate = true;
        }
    }
}
static void recalculate_trapezoids(std::vector<TimeBlock>& blocks)
{
    // GCodeProcessor.cpp:369-395
    for (size_t i = 0; i + 1 < blocks.size(); ++i) {
        if (blocks[i].flags.recalculate || blocks[i + 1].flags.recalculate) {
            blocks[i].feedrate_profile.exit = blocks[i + 1].feedrate_profile.entry;
            blocks[i].calculate_trapezoid();
            blocks[i].flags.recalculate = false;
        }
    }
    if (!blocks.empty()) {
        blocks.back().feedrate_profile.exit = blocks.back().safe_feedrate;
        blocks.back().calculate_trapezoid();
        blocks.back().flags.recalculate = false;
    }
}

struct MachineLimits {
    float max_speed[4]{ 0, 0, 0, 0 };
    float max_accel[4]{ 0, 0, 0, 0 };
    float max_jerk[4]{ 0, 0, 0, 0 };
    float junction_deviation{ 0.0f };
    float min_extruding{ 0.0f };
    float min_travel{ 0.0f };
};

struct State {
    // TimeMachine::State (GCodeProcessor.hpp:484-497)
    float feedrate;
    float safe_feedrate;
    AxisCoords axis_feedrate{};
    AxisCoords abs_axis_feedrate{};
    float enter_direction[3]{};
    float exit_direction[3]{};
};

struct CacheEntry {
    unsigned int id;
    float time;
};

class TimeMachine {
public:
    // GCodeProcessor.hpp TimeProcessor::Planner defaults
    static constexpr size_t refresh_threshold = 256;
    static constexpr size_t queue_size = 64;

    State curr{}, prev{};
    std::vector<TimeBlock> blocks;
    std::vector<std::pair<EMoveType, float>> m_additional_time_buffer;
    std::vector<CacheEntry> g1_times_cache;
    double time = 0.0;

    float acceleration = 0.0f, travel_acceleration = 0.0f, retract_acceleration = 0.0f;
    float max_acceleration = 0.0f, max_travel_acceleration = 0.0f, max_retract_acceleration = 0.0f;

    void set_acceleration(float value) {
        acceleration = (max_acceleration == 0.0f) ? value : std::min(value, max_acceleration);
    }
    void set_travel_acceleration(float value) {
        travel_acceleration = (max_travel_acceleration == 0.0f) ? value : std::min(value, max_travel_acceleration);
    }
    void set_retract_acceleration(float value) {
        retract_acceleration = (max_retract_acceleration == 0.0f) ? value : std::min(value, max_retract_acceleration);
    }

    // GCodeProcessor.cpp:399-416
    std::vector<std::pair<EMoveType, float>> merge_adjacent(const std::vector<std::pair<EMoveType, float>>& buffer) const {
        std::vector<std::pair<EMoveType, float>> merged;
        if (buffer.empty()) return merged;
        auto current_block = buffer.front();
        for (size_t idx = 1; idx < buffer.size(); ++idx) {
            const auto& next_block = buffer[idx];
            if (current_block.first == next_block.first)
                current_block.second += next_block.second;
            else {
                merged.push_back(current_block);
                current_block = next_block;
            }
        }
        merged.push_back(current_block);
        return merged;
    }

    // GCodeProcessor.cpp:418-596 calculate_time (result bookkeeping elided;
    // per-block dump added)
    void calculate_time(size_t keep_last_n_blocks, float additional_time, EMoveType target_move_type, bool is_final)
    {
        const bool drain_final = is_final && !m_additional_time_buffer.empty();
        if (blocks.size() < 2 && !drain_final) {
            if (additional_time > 0.0f)
                m_additional_time_buffer.emplace_back(target_move_type, additional_time);
            return;
        }
        std::vector<std::pair<EMoveType, float>> additional_buffer = m_additional_time_buffer;
        if (additional_time > 0.0f)
            additional_buffer.emplace_back(target_move_type, additional_time);
        additional_buffer = merge_adjacent(additional_buffer);

        for (int i = static_cast<int>(blocks.size()) - 1; i > 0; --i)
            planner_reverse_pass_kernel(blocks[i - 1], blocks[i]);
        for (size_t i = 0; i + 1 < blocks.size(); ++i)
            planner_forward_pass_kernel(blocks[i], blocks[i + 1]);
        recalculate_trapezoids(blocks);

        const size_t n_blocks_process = blocks.size() - keep_last_n_blocks;
        size_t additional_buffer_idx = 0;
        for (size_t i = 0; i < n_blocks_process; ++i) {
            const TimeBlock& block = blocks[i];
            float block_time = block.time();
            if (additional_buffer_idx < additional_buffer.size()) {
                const EMoveType buf_move_type = additional_buffer[additional_buffer_idx].first;
                if (buf_move_type == EMoveType::Noop || buf_move_type == block.move_type) {
                    block_time += additional_buffer[additional_buffer_idx].second;
                    ++additional_buffer_idx;
                }
            }
            time += double(block_time);
            g1_times_cache.push_back({ block.g1_line_id, float(time) });
            if (getenv("REPLAY_DUMP_BLOCKS"))
                std::fprintf(stderr, "%u %.8g %.8g %.8g %.8g %.8g %.8g %.8g\n",
                    block.g1_line_id, block.distance, block.feedrate_profile.cruise, block.acceleration,
                    block.feedrate_profile.entry, block.feedrate_profile.exit, block.safe_feedrate, block_time);
        }

        m_additional_time_buffer.clear();
        if (additional_buffer_idx < additional_buffer.size()) {
            if (is_final) {
                float leftover = 0.0f;
                for (size_t i = additional_buffer_idx; i < additional_buffer.size(); ++i)
                    leftover += additional_buffer[i].second;
                time += double(leftover);
            } else {
                m_additional_time_buffer.insert(m_additional_time_buffer.end(), additional_buffer.begin() + additional_buffer_idx, additional_buffer.end());
            }
        }

        blocks.erase(blocks.begin(), blocks.begin() + n_blocks_process);
    }
};

struct Machine {
    TimeMachine machine;
    MachineLimits limits;
};

int main(int argc, char** argv)
{
    if (argc != 3) {
        std::fprintf(stderr, "usage: replay <input.gcode> <limits-file>\n");
        return 2;
    }
    std::ifstream gcode(argv[1]);
    if (!gcode) { std::fprintf(stderr, "cannot open %s\n", argv[1]); return 2; }
    MachineLimits limits;
    float accel_init = 0, travel_accel_init = 0, retract_accel_init = 0;
    float max_accel_init = 0, max_travel_accel_init = 0, max_retract_accel_init = 0;
    {
        std::ifstream lf(argv[2]);
        if (!lf) { std::fprintf(stderr, "cannot open %s\n", argv[2]); return 2; }
        std::string key; double value;
        while (lf >> key >> value) {
            if (key == "max_speed_x") limits.max_speed[X] = float(value);
            else if (key == "max_speed_y") limits.max_speed[Y] = float(value);
            else if (key == "max_speed_z") limits.max_speed[Z] = float(value);
            else if (key == "max_speed_e") limits.max_speed[E] = float(value);
            else if (key == "max_accel_x") limits.max_accel[X] = float(value);
            else if (key == "max_accel_y") limits.max_accel[Y] = float(value);
            else if (key == "max_accel_z") limits.max_accel[Z] = float(value);
            else if (key == "max_accel_e") limits.max_accel[E] = float(value);
            else if (key == "max_jerk_x") limits.max_jerk[X] = float(value);
            else if (key == "max_jerk_y") limits.max_jerk[Y] = float(value);
            else if (key == "max_jerk_z") limits.max_jerk[Z] = float(value);
            else if (key == "max_jerk_e") limits.max_jerk[E] = float(value);
            else if (key == "junction_deviation") limits.junction_deviation = float(value);
            else if (key == "min_extruding") limits.min_extruding = float(value);
            else if (key == "min_travel") limits.min_travel = float(value);
            else if (key == "accel") accel_init = float(value);
            else if (key == "travel_accel") travel_accel_init = float(value);
            else if (key == "retract_accel") retract_accel_init = float(value);
            else if (key == "max_accel") max_accel_init = float(value);
            else if (key == "max_travel_accel") max_travel_accel_init = float(value);
            else if (key == "max_retract_accel") max_retract_accel_init = float(value);
        }
    }
    TimeMachine machine;
    machine.acceleration = accel_init;
    machine.travel_acceleration = travel_accel_init;
    machine.retract_acceleration = retract_accel_init;
    machine.max_acceleration = max_accel_init;
    machine.max_travel_acceleration = max_travel_accel_init;
    machine.max_retract_acceleration = max_retract_accel_init;

    bool absolute = true;
    bool e_relative = false;
    bool bbl_printer = getenv("REPLAY_BBL") != nullptr;
    bool measure_g29_time = false;
    AxisCoords start_position{ 0, 0, 0, 0 };
    AxisCoords end_position{ 0, 0, 0, 0 };
    float origin[4]{ 0, 0, 0, 0 };
    float m_feedrate = 0.0f;
    unsigned int g1_line_id = 0;

    auto minimum_feedrate = [&](float feedrate) {
        return std::max(feedrate, limits.min_extruding);
    };
    auto minimum_travel_feedrate = [&](float feedrate) {
        return std::max(feedrate, limits.min_travel);
    };

    auto process_G1 = [&](const std::array<std::optional<double>, 4>& axes, const std::optional<double>& feedrate) {
        auto move_type = [](const AxisCoords& delta_pos) {
            // GCodeProcessor.cpp:3846-3867 (m_wiping always false in replay)
            if (delta_pos[E] < 0.0f)
                return (delta_pos[X] != 0.0f || delta_pos[Y] != 0.0f || delta_pos[Z] != 0.0f) ? EMoveType::Travel : EMoveType::Retract;
            if (delta_pos[E] > 0.0f) {
                if (delta_pos[X] == 0.0f && delta_pos[Y] == 0.0f)
                    return (delta_pos[Z] == 0.0f) ? EMoveType::Unretract : EMoveType::Travel;
                else
                    return EMoveType::Extrude;
            }
            if (delta_pos[X] != 0.0f || delta_pos[Y] != 0.0f || delta_pos[Z] != 0.0f)
                return EMoveType::Travel;
            return EMoveType::Noop;
        };

        ++g1_line_id;
        for (unsigned char a = X; a <= E; ++a) {
            const auto& value = axes[a];
            if (value.has_value()) {
                bool is_relative = !absolute;
                if (Axis(a) == E && e_relative) is_relative = true;
                double ret = *value;
                end_position[a] = float(is_relative ? start_position[a] + ret : origin[a] + ret);
            } else
                end_position[a] = start_position[a];
        }
        if (feedrate.has_value())
            m_feedrate = float(*feedrate) / 60.0f;

        float max_abs_delta = 0.0f;
        AxisCoords delta_pos{};
        for (unsigned char a = X; a <= E; ++a) {
            delta_pos[a] = end_position[a] - start_position[a];
            max_abs_delta = std::max<float>(max_abs_delta, std::fabs(delta_pos[a]));
        }
        if (max_abs_delta == 0.0f) {
            start_position = end_position;
            return;
        }
        EMoveType type = move_type(delta_pos);

        // GCodeProcessor.cpp:3965-3972
        float sq_xyz_length = sqr(delta_pos[X]) + sqr(delta_pos[Y]) + sqr(delta_pos[Z]);
        float distance = (sq_xyz_length > 0.0f) ? std::sqrt(sq_xyz_length) : std::fabs(delta_pos[E]);
        float inv_distance = 1.0f / distance;
        bool extrusion_only = delta_pos[X] == 0.0f && delta_pos[Y] == 0.0f && delta_pos[Z] == 0.0f && delta_pos[E] != 0.0f;

        State& curr = machine.curr;
        State& prev = machine.prev;
        std::vector<TimeBlock>& blocks = machine.blocks;

        curr.feedrate = (delta_pos[E] == 0.0f) ? minimum_travel_feedrate(m_feedrate) : minimum_feedrate(m_feedrate);

        // enter/exit directions
        curr.enter_direction[0] = delta_pos[X];
        curr.enter_direction[1] = delta_pos[Y];
        curr.enter_direction[2] = delta_pos[Z];
        float norm = std::sqrt(sqr(curr.enter_direction[0]) + sqr(curr.enter_direction[1]) + sqr(curr.enter_direction[2]));
        if (!extrusion_only) {
            curr.enter_direction[0] /= norm;
            curr.enter_direction[1] /= norm;
            curr.enter_direction[2] /= norm;
        }
        curr.exit_direction[0] = curr.enter_direction[0];
        curr.exit_direction[1] = curr.enter_direction[1];
        curr.exit_direction[2] = curr.enter_direction[2];

        TimeBlock block;
        block.move_type = type;
        block.distance = distance;
        block.g1_line_id = g1_line_id;

        // centripetal limit (GCodeProcessor.cpp:4013-4033)
        if ((prev.exit_direction[0] != 0.0f || prev.exit_direction[1] != 0.0f) &&
            (curr.enter_direction[0] != 0.0f || curr.enter_direction[1] != 0.0f)) {
            float v1x = prev.exit_direction[0], v1y = prev.exit_direction[1];
            float v1n = std::sqrt(sqr(v1x) + sqr(v1y));
            v1x /= v1n; v1y /= v1n;
            float v2x = curr.enter_direction[0], v2y = curr.enter_direction[1];
            float v2n = std::sqrt(sqr(v2x) + sqr(v2y));
            v2x /= v2n; v2y /= v2n;
            float norm_diff = std::sqrt(sqr(v2x - v1x) + sqr(v2y - v1y));
            if (norm_diff < 0.5f && norm_diff > 0.00001f) {
                float dot = v1x * v2x + v1y * v2y;
                float cross = v1x * v2y - v1y * v2x;
                float angle = float(std::atan2(double(cross), double(dot)));
                float sin_theta_2 = std::sqrt((1.0f - std::cos(angle)) * 0.5f);
                float xy_len = std::sqrt(sqr(delta_pos[X]) + sqr(delta_pos[Y]));
                float r = xy_len * 0.5f / sin_theta_2;
                float acc = machine.acceleration;
                { curr.feedrate = std::min(curr.feedrate, std::sqrt(acc * r)); if (getenv("REPLAY_TRACE_G1") && curr.feedrate < 1.0f) std::fprintf(stderr, "CENTRI acc=%.2f r=%.8f nd=%.5f newf=%.5f\n", acc, r, norm_diff, curr.feedrate); }
            }
        }

        // min_feedrate_factor (GCodeProcessor.cpp:4036-4060)
        float min_feedrate_factor = 1.0f;
        for (unsigned char a = X; a <= E; ++a) {
            curr.axis_feedrate[a] = curr.feedrate * delta_pos[a] * inv_distance;
            curr.abs_axis_feedrate[a] = std::fabs(curr.axis_feedrate[a]);
            if (curr.abs_axis_feedrate[a] != 0.0f) {
                float axis_max_feedrate = limits.max_speed[a];
                if (axis_max_feedrate != 0.0f)
                    min_feedrate_factor = std::min<float>(min_feedrate_factor, axis_max_feedrate / curr.abs_axis_feedrate[a]);
            }
        }
        curr.feedrate *= min_feedrate_factor;
        if (getenv("REPLAY_TRACE_G1") && curr.feedrate < 1.0f) std::fprintf(stderr, "FACTOR f=%.6f af=%.4f,%.4f,%.4f,%.4f\n", min_feedrate_factor, curr.axis_feedrate[0], curr.axis_feedrate[1], curr.axis_feedrate[2], curr.axis_feedrate[3]);
        block.feedrate_profile.cruise = curr.feedrate;
        if (min_feedrate_factor < 1.0f) {
            for (unsigned char a = X; a <= E; ++a) {
                curr.axis_feedrate[a] *= min_feedrate_factor;
                curr.abs_axis_feedrate[a] *= min_feedrate_factor;
            }
        }

        // acceleration (GCodeProcessor.cpp:4062-4076)
        float acceleration =
            (type == EMoveType::Travel) ? machine.travel_acceleration :
            (extrusion_only ? machine.retract_acceleration : machine.acceleration);
        for (unsigned char a = X; a <= E; ++a) {
            float axis_max_acceleration = limits.max_accel[a];
            if (acceleration * std::fabs(delta_pos[a]) * inv_distance > axis_max_acceleration)
                acceleration = axis_max_acceleration / (std::fabs(delta_pos[a]) * inv_distance);
        }
        block.acceleration = acceleration;

        // safe feedrate (GCodeProcessor.cpp:4078-4086)
        // MarlinFirmware with a positive machine_max_junction_deviation derives
        // the per-axis jerk limit from the junction deviation and the current
        // machine acceleration clamped by the per-axis maximum
        // (GCodeProcessor.cpp:5798-5806, 5771-5792); classic M205 jerks apply
        // otherwise.
        float effective_max_jerk[4];
        for (unsigned char a = X; a <= E; ++a) {
            if (limits.junction_deviation > 0.0f) {
                float eff_acc = machine.acceleration;
                float axis_max_acc = limits.max_accel[a];
                if (axis_max_acc > 0.0f)
                    eff_acc = (eff_acc > 0.0f) ? std::min(eff_acc, axis_max_acc) : axis_max_acc;
                effective_max_jerk[a] = (eff_acc > 0.0f) ? std::sqrt(limits.junction_deviation * eff_acc * 2.5f) : 0.0f;
            } else {
                effective_max_jerk[a] = limits.max_jerk[a];
            }
        }
        curr.safe_feedrate = block.feedrate_profile.cruise;
        for (unsigned char a = X; a <= E; ++a) {
            float axis_max_jerk = effective_max_jerk[a];
            if (curr.abs_axis_feedrate[a] > axis_max_jerk)
                curr.safe_feedrate = std::min(curr.safe_feedrate, axis_max_jerk);
        }
        block.feedrate_profile.exit = curr.safe_feedrate;

        // vmax_junction (GCodeProcessor.cpp:4089-4164)
        static const float PREVIOUS_FEEDRATE_THRESHOLD = 0.0001f;
        float vmax_junction = curr.safe_feedrate;
        if (!blocks.empty() && prev.feedrate > PREVIOUS_FEEDRATE_THRESHOLD) {
            bool prev_speed_larger = prev.feedrate > block.feedrate_profile.cruise;
            float smaller_speed_factor = prev_speed_larger ? (block.feedrate_profile.cruise / prev.feedrate) : (prev.feedrate / block.feedrate_profile.cruise);
            vmax_junction = prev_speed_larger ? block.feedrate_profile.cruise : prev.feedrate;

            float v_factor = 1.0f;
            bool limited = false;

            // X branch: XYZ jerk via direction vectors
            {
                float exit_v[3] = { prev.feedrate * prev.exit_direction[0], prev.feedrate * prev.exit_direction[1], prev.feedrate * prev.exit_direction[2] };
                if (prev_speed_larger) { exit_v[0] *= smaller_speed_factor; exit_v[1] *= smaller_speed_factor; exit_v[2] *= smaller_speed_factor; }
                float entry_v[3] = { block.feedrate_profile.cruise * curr.enter_direction[0], block.feedrate_profile.cruise * curr.enter_direction[1], block.feedrate_profile.cruise * curr.enter_direction[2] };
                float jerk_v[3] = { std::fabs(entry_v[0] - exit_v[0]), std::fabs(entry_v[1] - exit_v[1]), std::fabs(entry_v[2] - exit_v[2]) };
                float max_xyz_jerk_v[3] = { effective_max_jerk[X], effective_max_jerk[Y], effective_max_jerk[Z] };
                for (int i = 0; i < 3; ++i) {
                    if (jerk_v[i] > max_xyz_jerk_v[i]) {
                        v_factor *= max_xyz_jerk_v[i] / jerk_v[i];
                        jerk_v[0] *= v_factor; jerk_v[1] *= v_factor; jerk_v[2] *= v_factor;
                        limited = true;
                    }
                }
            }
            // E axis branch
            {
                float v_exit = prev.axis_feedrate[E];
                float v_entry = curr.axis_feedrate[E];
                if (prev_speed_larger) v_exit *= smaller_speed_factor;
                if (limited) { v_exit *= v_factor; v_entry *= v_factor; }
                float jerk =
                    (v_exit > v_entry) ?
                        (((v_entry > 0.0f) || (v_exit < 0.0f)) ? (v_exit - v_entry) : std::max(v_exit, -v_entry)) :
                        (((v_entry < 0.0f) || (v_exit > 0.0f)) ? (v_entry - v_exit) : std::max(-v_exit, v_entry));
                float axis_max_jerk = effective_max_jerk[E];
                if (jerk > axis_max_jerk) {
                    v_factor *= axis_max_jerk / jerk;
                    limited = true;
                }
            }
            if (limited)
                vmax_junction *= v_factor;

            float vmax_junction_threshold = vmax_junction * 0.99f;
            if (prev.safe_feedrate > vmax_junction_threshold && curr.safe_feedrate > vmax_junction_threshold)
                vmax_junction = curr.safe_feedrate;
        }

        float v_allowable = max_allowable_speed(-acceleration, curr.safe_feedrate, block.distance);
        if (getenv("REPLAY_DEBUG_JUNCTION"))
            std::fprintf(stderr, "id=%u vmax=%.6g vmax_pre=%.6g prev_feed=%.6g prev_safe=%.6g curr_safe=%.6g\n",
                g1_line_id, std::min(vmax_junction, v_allowable), vmax_junction, prev.feedrate, prev.safe_feedrate, curr.safe_feedrate);
        block.feedrate_profile.entry = std::min(vmax_junction, v_allowable);
        block.max_entry_speed = vmax_junction;
        block.flags.nominal_length = (block.feedrate_profile.cruise <= v_allowable);
        block.flags.recalculate = true;
        block.safe_feedrate = curr.safe_feedrate;
        block.calculate_trapezoid();

        if (getenv("REPLAY_TRACE_G1") && std::fabs(distance - 0.20798) < 0.001) std::fprintf(stderr, "BLOCK d=%.5f cruise=%.5f mf=%.5f factor? entry=%.5f\n", distance, block.feedrate_profile.cruise, m_feedrate, block.feedrate_profile.entry);
        machine.prev = machine.curr;
        blocks.push_back(block);

        if (blocks.size() > TimeMachine::refresh_threshold)
            machine.calculate_time(TimeMachine::queue_size, 0.0f, EMoveType::Noop, false);

        start_position = end_position;
    };

    auto simulate_st_synchronize = [&](float additional_time) {
        machine.calculate_time(0, additional_time, EMoveType::Noop, false); if (additional_time > 0) std::fprintf(stderr, "DELAY %.3f at id=%u\n", additional_time, g1_line_id);
    };

    std::string line;
    while (std::getline(gcode, line)) {
        std::string code = line.substr(0, line.find(';'));
        std::istringstream iss(code);
        std::string word;
        iss >> word;
        if (word.empty()) continue;
        auto word_value = [&](char axis) -> std::optional<double> {
            std::string rest = code;
            for (size_t pos = 0; pos + 1 < rest.size(); ++pos) {
                if (rest[pos] == axis && (pos == 0 || rest[pos - 1] == ' ' || rest[pos - 1] == '\t')) {
                    std::string num;
                    size_t s = pos + 1;
                    while (s < rest.size() && (isdigit((unsigned char)rest[s]) || rest[s] == '.' || rest[s] == '-' || rest[s] == '+'))
                        num += rest[s++];
                    if (!num.empty()) return std::atof(num.c_str());
                }
            }
            return std::nullopt;
        };
        if (word == "G0" || word == "G1") {
            std::array<std::optional<double>, 4> axes{ std::nullopt, std::nullopt, std::nullopt, std::nullopt };
            axes[X] = word_value('X');
            axes[Y] = word_value('Y');
            axes[Z] = word_value('Z');
            axes[E] = word_value('E');
            if (getenv("REPLAY_TRACE_G1"))
                std::fprintf(stderr, "G1 id=%u line=[%s] pos=%.3f,%.3f,%.3f F=%.2f\n", g1_line_id + 1, code.c_str(),
                    start_position[X], start_position[Y], start_position[Z], m_feedrate);
            process_G1(axes, word_value('F'));
        } else if (word == "G2" || word == "G3") {
            const bool clockwise = (word == "G2");
            // GCodeProcessor.cpp:4548-4846 process_G2_G3 (IJ fitting, legacy
            // non-MarlinFirmware discretization)
            auto ijc = word_value('I');
            auto jjc = word_value('J');
            auto rword = word_value('R');
            if (!ijc.has_value() && !jjc.has_value() && !(rword.has_value() && *rword != 0.0))
                continue;
            float rel_cx, rel_cy;
            AxisCoords end_pos = start_position;
            {
                std::array<std::optional<double>, 4> axes{ std::nullopt, std::nullopt, std::nullopt, std::nullopt };
                axes[X] = word_value('X');
                axes[Y] = word_value('Y');
                axes[Z] = word_value('Z');
                axes[E] = word_value('E');
                for (unsigned char a = X; a <= E; ++a) {
                    const auto& value = axes[a];
                    if (value.has_value()) {
                        bool is_relative = !absolute;
                        if (Axis(a) == E && e_relative) is_relative = true;
                        end_pos[a] = float(is_relative ? start_position[a] + *value : origin[a] + *value);
                    }
                }
            }
            double cx = start_position[X] + rel_cx;
            double cy = start_position[Y] + rel_cy;
            if (rword.has_value() && *rword != 0.0) {
                // ArcWelder::arc_center (ArcWelder.hpp:23-42)
                double vx = end_pos[X] - start_position[X], vy = end_pos[Y] - start_position[Y];
                double q2 = vx * vx + vy * vy;
                double t2 = (*rword) * (*rword) / q2 - 0.25;
                double t = t2 > 0 ? std::sqrt(t2) : 0.0;
                double mx = 0.5 * (start_position[X] + end_pos[X]), my = 0.5 * (start_position[Y] + end_pos[Y]);
                double vpx = -vy * t, vpy = vx * t;
                bool ccw = !clockwise;
                cx = ((*rword) > 0.0) == ccw ? mx + vpx : mx - vpx;
                cy = ((*rword) > 0.0) == ccw ? my + vpy : my - vpy;
                rel_cx = float(cx - start_position[X]);
                rel_cy = float(cy - start_position[Y]);
            } else {
                rel_cx = float(ijc.value_or(0.0));
                rel_cy = float(jjc.value_or(0.0));
            }
            double rsx = start_position[X] - cx, rsy = start_position[Y] - cy;
            double rex = end_pos[X] - cx, rey = end_pos[Y] - cy;
            double angle;
            bool full_circle = std::fabs(end_pos[X] - start_position[X]) < 1e-4 && std::fabs(end_pos[Y] - start_position[Y]) < 1e-4;
            if (full_circle)
                angle = 2.0 * M_PI;
            else {
                angle = std::atan2(rsx * rey - rsy * rex, rsx * rex + rsy * rey);
                if (angle < 0.0) angle += 2.0 * M_PI;
                if (clockwise) angle -= 2.0 * M_PI;
            }
            double dz = end_pos[Z] - start_position[Z];
            double radius = std::sqrt(rsx * rsx + rsy * rsy);
            double length = angle * radius;
            double travel_length = std::sqrt(length * length + dz * dz);
            if (travel_length < 0.001) { start_position = end_pos; continue; }
            std::optional<float> arc_feedrate;
            if (auto f = word_value('F')) { arc_feedrate = float(*f); if (getenv("REPLAY_TRACE_G1")) std::fprintf(stderr, "ARC F=%.1f segments=%zu r=%.3f angle=%.4f\n", *f, 0, radius, angle); }
            std::optional<float> extrusion;
            if (auto ev = word_value('E')) extrusion = end_pos[E] - start_position[E];
            // GCodeProcessor.cpp:4712-4726: gcfMarlinFirmware plan_arc
            // discretization (MAX_ARC_DEVIATION 0.02, min 50 segments/s,
            // segment length clamped to [0.1, 2.0] mm). The legacy
            // ArcWelder rule (tolerance 0.0125) stays available via
            // REPLAY_LEGACY_ARCS=1 for non-MarlinFirmware flavors.
            size_t segments;
            if (getenv("REPLAY_SINGLE_ARCS")) {
                // Diagnostic: treat each arc as one move (arc-length distance).
                segments = 1;
            } else if (getenv("REPLAY_LEGACY_ARCS")) {
                static const double gcode_arc_tolerance = 0.0125;
                double d = radius - gcode_arc_tolerance;
                if (d < 1e-4) {
                    segments = (std::fabs(angle) < M_PI || radius * (1.0 + cos(M_PI - 0.5 * std::fabs(angle))) < gcode_arc_tolerance) ? 1 : 2;
                } else
                    segments = size_t(std::ceil(std::fabs(angle) / (2.0 * std::acos(d / radius))));
                if (segments == 0) segments = 1;
            } else {
                static const float MAX_ARC_DEVIATION = 0.02f;
                static const float MIN_ARC_SEGMENTS_PER_SEC = 50;
                static const float MIN_ARC_SEGMENT_MM = 0.1f;
                static const float MAX_ARC_SEGMENT_MM = 2.0f;
                const float feedrate_mm_s = arc_feedrate.has_value() ? *arc_feedrate / 60.0f : m_feedrate;
                const float radius_mm = float(radius);
                float segment_mm = std::min(std::sqrt(8.0f * radius_mm * MAX_ARC_DEVIATION), feedrate_mm_s / MIN_ARC_SEGMENTS_PER_SEC);
                segment_mm = std::max(std::min(segment_mm, MAX_ARC_SEGMENT_MM), MIN_ARC_SEGMENT_MM);
                const float flat_mm = radius_mm * float(std::fabs(angle));
                segments = std::max<size_t>(size_t(flat_mm / segment_mm + 0.8f), size_t(1));
            }
            const double inv_segment = 1.0 / double(segments);
            const double theta_per_segment = angle * inv_segment;
            const double z_per_segment = dz * inv_segment;
            const double extruder_per_segment = extrusion.has_value() ? double(*extrusion) * inv_segment : 0.0;
            const double sq_theta = theta_per_segment * theta_per_segment;
            const double cos_T = 1.0 - 0.5 * sq_theta;
            const double sin_T = theta_per_segment - sq_theta * theta_per_segment / 6.0;
            AxisCoords arc_target = start_position;
            static const size_t N_ARC_CORRECTION = 25;
            double relx = rsx, rely = rsy;
            size_t count = N_ARC_CORRECTION;
            auto emit_seg = [&](const AxisCoords& target, std::optional<float> seg_feedrate) {
                // adjust_target + internal_only_g1_line: absolute or relative
                std::array<std::optional<double>, 4> axes{ std::nullopt, std::nullopt, std::nullopt, std::nullopt };
                if (absolute) {
                    axes[X] = target[X]; axes[Y] = target[Y]; axes[Z] = target[Z];
                } else {
                    axes[X] = target[X] - start_position[X]; axes[Y] = target[Y] - start_position[Y];
                    axes[Z] = target[Z] - start_position[Z];
                }
                // adjust_target sets E only when the ARC line carried an
                // E word (GCodeProcessor.cpp:4701 `if (extrusion.has_value())`);
                // E-less arcs leave the axis unset so the position carries.
                if (extrusion.has_value())
                    axes[E] = e_relative ? std::optional<double>(target[E] - start_position[E]) : std::optional<double>(target[E]);
                process_G1(axes, seg_feedrate.has_value() ? std::optional<double>(double(*seg_feedrate)) : std::nullopt);
            };
            for (size_t i = 1; i < segments; ++i) {
                if (count-- == 0) {
                    const double cos_Ti = std::cos(i * theta_per_segment);
                    const double sin_Ti = std::sin(i * theta_per_segment);
                    relx = -double(rel_cx) * cos_Ti + double(rel_cy) * sin_Ti;
                    rely = -double(rel_cx) * sin_Ti - double(rel_cy) * cos_Ti;
                    count = N_ARC_CORRECTION;
                } else {
                    const double r_axisi = relx * sin_T + rely * cos_T;
                    relx = relx * cos_T - rely * sin_T;
                    rely = r_axisi;
                }
                arc_target[X] = float(cx + relx);
                arc_target[Y] = float(cy + rely);
                arc_target[Z] = float(arc_target[Z] + z_per_segment);
                arc_target[E] = float(arc_target[E] + extruder_per_segment);
                if (i == 1 && getenv("REPLAY_TRACE_G1")) std::fprintf(stderr, "SEG1 feed=%s target=%.4f,%.4f start=%.4f,%.4f m_f=%.4f\n", arc_feedrate.has_value() ? std::to_string(*arc_feedrate).c_str() : "none", arc_target[X], arc_target[Y], start_position[X], start_position[Y], m_feedrate);
                emit_seg(arc_target, (i == 1) ? arc_feedrate : std::nullopt);
            }
            emit_seg(end_pos, (segments == 1) ? arc_feedrate : std::nullopt);
        } else if (word == "G29") {
            // GCodeProcessor.cpp:4856-4868 (BBL gate via M622 J1/M623)
            if ((getenv("REPLAY_G29") && !bbl_printer) || (bbl_printer && measure_g29_time))
                simulate_st_synchronize(260.0f);
        } else if (word == "M191") {
            if (auto s = word_value('S'); s.has_value() && *s > 40.0)
                simulate_st_synchronize(720.0f);
        } else if (word == "SYNC") {
            if (auto t = word_value('T'))
                simulate_st_synchronize(float(*t));
        } else if (word == "M702") {
            if (code.find('C') != std::string::npos)
                simulate_st_synchronize(0.0f); // filament unload time; zero for non-MMU
        } else if (word == "M622") {
            if (auto j = word_value('J'); j.has_value() && std::round(*j) == 1.0)
                measure_g29_time = true;
        } else if (word == "M623") {
            measure_g29_time = false;
        } else if (word == "G4" || word == "M400") {
            float s = float(word_value('S').value_or(0.0));
            float p = float(word_value('P').value_or(0.0));
            if (word_value('S').has_value() || word_value('P').has_value())
                simulate_st_synchronize(s + p * 0.001f);
        } else if (word == "G28") {
            // upstream reparses `G28 X..` with the axis word PRESENCE
            // (GCodeProcessor.cpp:4915-4938), not its value
            auto has_word = [&](char axis) {
                for (size_t pos = 0; pos < code.size(); ++pos)
                    if (code[pos] == axis && (pos == 0 || isspace((unsigned char)code[pos - 1])))
                        return true;
                return false;
            };
            std::array<std::optional<double>, 4> axes{ std::nullopt, std::nullopt, std::nullopt, std::nullopt };
            if (has_word('X')) axes[X] = 0.0;
            if (has_word('Y')) axes[Y] = 0.0;
            if (has_word('Z')) axes[Z] = 0.0;
            if (!axes[X].has_value() && !axes[Y].has_value() && !axes[Z].has_value()) {
                axes[X] = 0.0; axes[Y] = 0.0; axes[Z] = 0.0;
            }
            process_G1(axes, std::nullopt);
        } else if (word == "G90") absolute = true;
        else if (word == "G91") absolute = false;
        else if (word == "M82") e_relative = false;
        else if (word == "M83") e_relative = true;
        else if (word == "G92") {
            // GCodeProcessor.cpp:4951+ (origin reset)
            bool any = false;
            if (word_value('X').has_value()) { origin[X] = end_position[X] - float(*word_value('X')); any = true; }
            if (word_value('Y').has_value()) { origin[Y] = end_position[Y] - float(*word_value('Y')); any = true; }
            if (word_value('Z').has_value()) { origin[Z] = end_position[Z] - float(*word_value('Z')); any = true; }
            if (word_value('E').has_value()) { origin[E] = end_position[E] - float(*word_value('E')); any = true; }
            if (!any) { origin[E] = end_position[E]; start_position[E] = end_position[E]; }
        } else if (word == "M204") {
            if (auto s = word_value('S')) { machine.set_acceleration(float(*s)); machine.set_travel_acceleration(float(*s)); if (auto t = word_value('T')) machine.set_retract_acceleration(float(*t)); }
            else {
                if (auto p = word_value('P')) machine.set_acceleration(float(*p));
                if (auto r = word_value('R')) machine.set_retract_acceleration(float(*r));
                if (auto t = word_value('T')) machine.set_travel_acceleration(float(*t));
            }
        } else if (word == "M201") {
            if (auto v = word_value('X')) limits.max_accel[X] = float(*v);
            if (auto v = word_value('Y')) limits.max_accel[Y] = float(*v);
            if (auto v = word_value('Z')) limits.max_accel[Z] = float(*v);
            if (auto v = word_value('E')) limits.max_accel[E] = float(*v);
        } else if (word == "M203") {
            // RepRapFirmware expresses M203 in mm/min; Marlin/Klipper/Smoothie
            // in mm/s (`GCodeProcessor.cpp:5184` factor). This replay assumes
            // the RepRap form when the values exceed plausible mm/s.
            auto factor = [](double v) { return v > 1000.0 ? 1.0 / 60.0 : 1.0; };
            if (auto v = word_value('X')) limits.max_speed[X] = float(*v * factor(*v));
            if (auto v = word_value('Y')) limits.max_speed[Y] = float(*v * factor(*v));
            if (auto v = word_value('Z')) limits.max_speed[Z] = float(*v * factor(*v));
            if (auto v = word_value('E')) limits.max_speed[E] = float(*v * factor(*v));
        } else if (word == "M205") {
            if (auto v = word_value('X')) { limits.max_jerk[X] = float(*v); limits.max_jerk[Y] = float(*v); }
            if (auto v = word_value('Y')) limits.max_jerk[Y] = float(*v);
            if (auto v = word_value('Z')) limits.max_jerk[Z] = float(*v);
            if (auto v = word_value('E')) limits.max_jerk[E] = float(*v);
        } else if (word == "SET_VELOCITY_LIMIT") {
            // GCodeProcessor.cpp:5269-5316
            for (size_t pos = code.find("SQUARE_CORNER_VELOCITY"); pos != std::string::npos; pos = code.find("SQUARE_CORNER_VELOCITY", pos + 1)) {
                std::string num;
                size_t s = code.find('=', pos);
                if (s == std::string::npos) break;
                s++;
                size_t e = s;
                while (e < code.size() && (isdigit((unsigned char)code[e]) || code[e] == '.')) num += code[e++];
                if (!num.empty()) { limits.max_jerk[X] = std::atof(num.c_str()); limits.max_jerk[Y] = std::atof(num.c_str()); }
                break;
            }
            for (size_t pos = code.find("ACCEL"); pos != std::string::npos; pos = code.find("ACCEL", pos + 1)) {
                // skip ACCEL_TO_DECEL (requires '=' right after optional spaces)
                std::string num;
                size_t s = pos + 5;
                while (s < code.size() && (code[s] == ' ')) s++;
                if (s >= code.size() || code[s] != '=') continue;
                s++;
                size_t e = s;
                while (e < code.size() && (isdigit((unsigned char)code[e]) || code[e] == '.')) num += code[e++];
                if (!num.empty()) {
                    float v = std::atof(num.c_str());
                    machine.set_acceleration(v);
                    machine.set_travel_acceleration(v);
                }
                break;
            }
            for (size_t pos = code.find("VELOCITY"); pos != std::string::npos; pos = code.find("VELOCITY", pos + 1)) {
                // SQUARE_CORNER_VELOCITY already handled; skip it
                if (pos >= 19 && code.compare(pos - 19, 19, "SQUARE_CORNER_VELOC") == 0) continue;
                std::string num;
                size_t s = code.find('=', pos);
                if (s == std::string::npos) break;
                s++;
                size_t e = s;
                while (e < code.size() && (isdigit((unsigned char)code[e]) || code[e] == '.')) num += code[e++];
                if (!num.empty()) { limits.max_speed[X] = std::atof(num.c_str()); limits.max_speed[Y] = std::atof(num.c_str()); }
                break;
            }
        }
    }
    machine.calculate_time(0, 0.0f, EMoveType::Noop, true);

    for (const auto& entry : machine.g1_times_cache)
        std::printf("%u %.6f\n", entry.id, entry.time);
    std::printf("TOTAL %.6f\n", machine.time);
    return 0;
}
