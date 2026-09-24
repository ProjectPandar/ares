let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/CoolingBuffer.cpp \
        --replace-fail '    return elapsed_time_total0;
}' '    if (getenv("ORCA_DUMP_SLOW") != nullptr) { static std::mutex sb; sb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SLOW"), "a"); if (f) { for (PerExtruderAdjustments &adj : per_extruder_adjustments) { fprintf(f, "SLOW ex=%d n=%u total=%.6f max=%.6f tgt=%.6f nonadj=%.6f\n", (int)(&adj - per_extruder_adjustments.data()), (unsigned)adj.lines.size(), adj.time_total, adj.time_maximum, adj.slow_down_layer_time, adj.time_non_adjustable); for (const CoolingLine &line : adj.lines) fprintf(f, "SLW t=%x len=%.6f feed=%.4f time=%.6f tmax=%.6f\n", line.type, line.length, line.feedrate, line.time, line.time_max); } fclose(f); } sb.unlock(); }
    return elapsed_time_total0;
}'
    '';
  })
