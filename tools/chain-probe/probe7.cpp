// probe7: feed a raw (marker-bearing) layer text through the vendored
// upstream CoolingBuffer to observe its slowdown solve directly.
//
// Usage: probe7 <raw-layers.txt>
// The input is split at ";LAYER_CHANGE" boundaries; each layer is fed with
// flush=true (single-extruder approximation, Ginger G1 config hardcoded).

#include "cbshim.hpp"
#include <cstdio>
#include <fstream>
#include <iostream>
#include <sstream>

using namespace Slic3r;

int main(int argc, char **argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: probe7 <raw-layers.txt>\n"); return 1; }
    std::ifstream in(argv[1]);
    std::stringstream buf;
    buf << in.rdbuf();
    std::string text = buf.str();

    GCode gcodegen;
    // Ginger G1 1.2 nozzle (case-2bDWHY)
    gcodegen.cfg.slow_down_for_layer_cooling.values = {true};
    gcodegen.cfg.slow_down_layer_time.values = {50.f};
    gcodegen.cfg.slow_down_min_speed.values = {1.f};
    gcodegen.cfg.dont_slow_down_outer_wall.values = {false};
    gcodegen.cfg.use_relative_e_distances = true;
    gcodegen.cfg.cooling.values = {true};
    gcodegen.cfg.fan_cooling_layer_time.values = {0.f};

    CoolingBuffer cb(gcodegen);

    size_t layer = 0;
    size_t pos = 0;
    while (true) {
        size_t next = text.find(";LAYER_CHANGE", pos);
        std::string chunk;
        if (next == std::string::npos) {
            chunk = text.substr(pos);
            if (chunk.empty()) break;
            std::string out = cb.process_layer(std::move(chunk), layer++, true);
            std::fputs(out.c_str(), stdout);
            break;
        }
        chunk = text.substr(pos, next - pos);
        if (!chunk.empty()) {
            std::string out = cb.process_layer(std::move(chunk), layer++, true);
            std::fputs(out.c_str(), stdout);
        }
        pos = next + 1;
    }
    return 0;
}
