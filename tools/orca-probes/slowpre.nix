let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/CoolingBuffer.cpp \
        --replace-fail '                extruder_range_slow_down_non_proportional(cur_begin, by_slowdown_time.end(), slow_down_layer_time - total);' '                if (getenv("ORCA_DUMP_SLOWPRE") != nullptr) { static std::mutex pb; pb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SLOWPRE"), "a"); if (f) { fprintf(f, "PRE deficit=%.6f nonadj0=%.6f\n", slow_down_layer_time - total, elapsed_time_total0); for (PerExtruderAdjustments *adj : by_slowdown_time) if (adj >= *cur_begin) for (size_t i = 0; i < adj->n_lines_adjustable; ++i) fprintf(f, "PL len=%.6f feed=%.6f time=%.6f tmax=%.6f\n", adj->lines[i].length, adj->lines[i].feedrate, adj->lines[i].time, adj->lines[i].time_max); fclose(f); } pb.unlock(); }
                extruder_range_slow_down_non_proportional(cur_begin, by_slowdown_time.end(), slow_down_layer_time - total);'
    '';
  })
