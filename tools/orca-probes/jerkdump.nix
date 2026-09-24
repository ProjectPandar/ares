let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail '        block.safe_feedrate = curr.safe_feedrate;' '        block.safe_feedrate = curr.safe_feedrate;
        if (getenv("ORCA_DUMP_JERK") != nullptr && block.g1_line_id >= 67 && block.g1_line_id <= 76) { static std::mutex jr; jr.lock(); FILE* f = fopen(getenv("ORCA_DUMP_JERK"), "a"); if (f) { Vec3f j = get_xyz_max_jerk(static_cast<PrintEstimatedStatistics::ETimeMode>(i)); fprintf(f, "JERK id=%u jerk=(%.6f,%.6f,%.6f) ejerk=%.6f safe=%.6f vmaxj=%.6f vallow=%.6f entry=%.6f\n", block.g1_line_id, j.x(), j.y(), j.z(), get_axis_max_jerk(static_cast<PrintEstimatedStatistics::ETimeMode>(i), E), curr.safe_feedrate, vmax_junction, v_allowable, block.feedrate_profile.entry); fclose(f); } jr.unlock(); }'
    '';
  })
