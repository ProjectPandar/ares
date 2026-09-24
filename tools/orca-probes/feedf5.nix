let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail "m_feedrate = (*feedrate) * MMMIN_TO_MMSEC;" "m_feedrate = (*feedrate) * MMMIN_TO_MMSEC; if (getenv(\"ORCA_DUMP_F3\") != nullptr) { FILE* f3 = fopen(getenv(\"ORCA_DUMP_F3\"), \"a\"); if (f3) { fprintf(f3, \"linef=%.6f m_feedrate=%.9f\n\", (*feedrate), m_feedrate); fclose(f3); } }"
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail "g1_times_cache.push_back({ block.g1_line_id, block.remaining_internal_g1_lines, float(time) });" "if (getenv(\"ORCA_DUMP_AF\") != nullptr) { FILE* f4 = fopen(getenv(\"ORCA_DUMP_AF\"), \"a\"); if (f4) { fprintf(f4, \"%u feed=%.9f dist=%.9f afx=%.9f afy=%.9f cruise=%.9f\n\", block.g1_line_id, curr.feedrate, block.distance, curr.axis_feedrate[0], curr.axis_feedrate[1], block.feedrate_profile.cruise); fclose(f4); } } g1_times_cache.push_back({ block.g1_line_id, block.remaining_internal_g1_lines, float(time) });"
    '';
  })
