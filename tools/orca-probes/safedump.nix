let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail 'block.feedrate_profile.exit = curr.safe_feedrate;' 'if (getenv("ORCA_DUMP_SAFE") != nullptr) { static std::mutex ms; ms.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SAFE"), "a"); if (f) { fprintf(f, "id=%u cruise=%.4f dE=%.6f feedrate=%.4f afE=%.6f afX=%.4f afY=%.4f afZ=%.4f dist=%.4f\\n", block.g1_line_id, block.feedrate_profile.cruise, delta_pos[E], curr.feedrate, curr.abs_axis_feedrate[3], curr.abs_axis_feedrate[0], curr.abs_axis_feedrate[1], curr.abs_axis_feedrate[2], distance); fclose(f); } ms.unlock(); }
        block.feedrate_profile.exit = curr.safe_feedrate;'
    '';
  })
