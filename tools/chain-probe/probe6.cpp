// Probe 6: run the vendored upstream PressureEqualizer (peq.hpp/peq.cpp
// shim) on an ARES_DUMP_PEINPUT layer file and print the rewrite — the
// byte-level oracle for the clamp-decision divergence.
//
// The dump contains per-layer sections separated by "=== LAYER ===" lines;
// each section is fed as its own process_layer call (with a final NOP to
// flush), matching ares's per-layer equalizer invocation order.
#include "peq.hpp"

#include <cstdio>
#include <fstream>
#include <sstream>

int main(int argc, char **argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: probe6 <peinput-layer-file> [slope]\n");
        return 2;
    }
    std::ifstream in(argv[1]);
    std::stringstream buffer;
    buffer << in.rdbuf();

    Slic3r::GCodeConfig config;
    config.max_volumetric_extrusion_rate_slope.value = argc > 2 ? std::atof(argv[2]) : 100.0;
    config.max_volumetric_extrusion_rate_slope_segment_length.value = 3.0;
    config.extrusion_rate_smoothing_external_perimeter_only.value = false;
    config.use_relative_e_distances.value = 1;
    config.filament_diameter.values = {1.12838};

    Slic3r::PressureEqualizer equalizer(config);

    const std::string text = buffer.str();
    const std::string sep = "=== LAYER ===\n";
    size_t pos = 0;
    while (true) {
        size_t next = text.find(sep, pos);
        std::string chunk;
        if (next == std::string::npos) {
            chunk = text.substr(pos);
        } else {
            chunk = text.substr(pos, next - pos);
        }
        if (!chunk.empty()) {
            Slic3r::LayerResult layer;
            layer.gcode = chunk;
            layer.nop_layer_result = false;
            Slic3r::LayerResult nop = Slic3r::LayerResult::make_nop_layer_result();
            // process_layer returns the PREVIOUS layer once buffered; print
            // whatever comes back so ordering matches the live pipeline.
            Slic3r::LayerResult out = equalizer.process_layer(std::move(layer));
            if (!out.gcode.empty())
                std::fwrite(out.gcode.data(), 1, out.gcode.size(), stdout);
            Slic3r::LayerResult out2 = equalizer.process_layer(std::move(nop));
            if (!out2.gcode.empty())
                std::fwrite(out2.gcode.data(), 1, out2.gcode.size(), stdout);
        }
        if (next == std::string::npos)
            break;
        pos = next + sep.size();
    }
    return 0;
}
