let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail '        //BBS: update curr.feedrate' '        if (getenv("ORCA_DUMP_FACTOR") != nullptr) { static std::mutex fb; fb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_FACTOR"), "a"); if (f) { fprintf(f, "FACT feed=%.6f factor=%.9f max=[%.6f %.6f %.6f %.6f] axf=[%.6f %.6f %.6f %.6f]\n", curr.feedrate, min_feedrate_factor, get_axis_max_feedrate((PrintEstimatedStatistics::ETimeMode)i, X), get_axis_max_feedrate((PrintEstimatedStatistics::ETimeMode)i, Y), get_axis_max_feedrate((PrintEstimatedStatistics::ETimeMode)i, Z), get_axis_max_feedrate((PrintEstimatedStatistics::ETimeMode)i, E), curr.axis_feedrate[0], curr.axis_feedrate[1], curr.axis_feedrate[2], curr.axis_feedrate[3]); fclose(f); } fb.unlock(); }
        //BBS: update curr.feedrate'
    '';
  })
