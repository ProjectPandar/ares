let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail "block.feedrate_profile.cruise = curr.feedrate;" "if (getenv(\"ORCA_DUMP_CHAIN\") != nullptr) { FILE* f5 = fopen(getenv(\"ORCA_DUMP_CHAIN\"), \"a\"); if (f5) { fprintf(f5, \"%u prefeed=%.9f dist=%.9f inv=%.9f dx=%.9f afx=%.9f factor=%.9f cruise=%.9f maxX=%.9f\n\", m_g1_line_id, min_feedrate_factor > 0 ? curr.feedrate / min_feedrate_factor : curr.feedrate, distance, inv_distance, delta_pos[0], curr.axis_feedrate[0], min_feedrate_factor, curr.feedrate, get_axis_max_feedrate(static_cast<PrintEstimatedStatistics::ETimeMode>(0), static_cast<Axis>(0))); fclose(f5); } } block.feedrate_profile.cruise = curr.feedrate;"
    '';
  })
