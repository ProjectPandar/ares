let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail '        float block_time = block.time();' '        if (getenv("ORCA_DUMP_TB2") != nullptr && block.g1_line_id >= 67 && block.g1_line_id <= 76) { static std::mutex tb; tb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_TB2"), "a"); if (f) { fprintf(f, "TB id=%u dist=%.6f prof=[e=%.6f c=%.6f x=%.6f] trap=[cu=%.6f au=%.6f da=%.6f] accel=%.6f safe=%.6f time=%.9f\n", block.g1_line_id, block.distance, block.feedrate_profile.entry, block.feedrate_profile.cruise, block.feedrate_profile.exit, block.trapezoid.cruise_feedrate, block.trapezoid.accelerate_until, block.trapezoid.decelerate_after, block.acceleration, block.safe_feedrate, block.time()); fclose(f); } tb.unlock(); }
        float block_time = block.time();'
    '';
  })
