let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/CoolingBuffer.cpp \
        --replace-fail '    for (auto cur_begin = by_slowdown_time.begin(); cur_begin != by_slowdown_time.end(); ++ cur_begin) {' '    if (getenv("ORCA_DUMP_SLOWALL") != nullptr) { static std::mutex pb; pb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SLOWALL"), "a"); if (f) { fprintf(f, "ALL n=%zu\n", per_extruder_adjustments[0].lines.size()); size_t k = 0; for (const CoolingLine &line : per_extruder_adjustments[0].lines) fprintf(f, "AL %zu type=%d len=%.9g feed=%.9g time=%.9g adj=%d\n", k++, (int)line.type, line.length, line.feedrate, line.time, (int)((line.type & CoolingLine::TYPE_ADJUSTABLE) != 0)); fclose(f); } pb.unlock(); }
    for (auto cur_begin = by_slowdown_time.begin(); cur_begin != by_slowdown_time.end(); ++ cur_begin) {'
    '';
  })
