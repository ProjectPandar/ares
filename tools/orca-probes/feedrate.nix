let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail 'g1_times_cache.push_back({ block.g1_line_id, block.remaining_internal_g1_lines, float(time) });' 'if (getenv("ORCA_DUMP_FEED") != nullptr) { static std::mutex mf; mf.lock(); FILE* f = fopen(getenv("ORCA_DUMP_FEED"), "a"); if (f) { fprintf(f, "%u feed=%.9f afx=%.9f afy=%.9f afz=%.9f afe=%.9f cruise=%.9f\\n", block.g1_line_id, curr.feedrate, curr.axis_feedrate[0], curr.axis_feedrate[1], curr.axis_feedrate[2], curr.axis_feedrate[3], block.feedrate_profile.cruise); fclose(f); } mf.unlock(); } g1_times_cache.push_back({ block.g1_line_id, block.remaining_internal_g1_lines, float(time) });'
    '';
  })
