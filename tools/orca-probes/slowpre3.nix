let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/CoolingBuffer.cpp \
        --replace-fail 'adj.sort_lines_by_decreasing_feedrate();' 'if (getenv("ORCA_DUMP_SLOWPRE2") != nullptr) { static std::mutex qb; qb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SLOWPRE2"), "a"); if (f) { fprintf(f, "PRESORT2 n=%zu\n", adj.lines.size()); size_t q = 0; for (const CoolingLine &line : adj.lines) fprintf(f, "PL %zu type=%d len=%.9g feed=%.9g time=%.9g tmax=%.9g\n", q++, (int)line.type, line.length, line.feedrate, line.time, line.time_max); fclose(f); } qb.unlock(); }
            adj.sort_lines_by_decreasing_feedrate();'
    '';
  })
