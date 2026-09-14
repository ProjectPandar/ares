// Probe 6: run the vendored upstream PressureEqualizer (peq.hpp/peq.cpp
// shim) on an ARES_DUMP_PEINPUT layer file and print the rewrite — the
// byte-level oracle for the clamp-decision divergence.
#include "peq.hpp"

#include <cstdio>
#include <fstream>
#include <sstream>

int main(int argc, char **argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: probe6 <peinput-layer-file>\n");
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

    Slic3r::LayerResult layer;
    layer.gcode = buffer.str();
    layer.nop_layer_result = false;
    // First call buffers; a NOP call flushes the previous layer.
    equalizer.process_layer(std::move(layer));
    Slic3r::LayerResult nop = Slic3r::LayerResult::make_nop_layer_result();
    Slic3r::LayerResult out = equalizer.process_layer(std::move(nop));

    std::fwrite(out.gcode.data(), 1, out.gcode.size(), stdout);
    return 0;
}
