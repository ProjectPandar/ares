let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail 'if (last_exported_main[i] != to_export_main) {' 'if (getenv("ORCA_DUMP_M73EMIT") != nullptr) { static std::mutex mm; mm.lock(); FILE* f = fopen(getenv("ORCA_DUMP_M73EMIT"), "a"); if (f) { fprintf(f, "EMIT id=%u pct=%d rem=%d elapsed=%.6f total=%.6f\\n", g1_lines_counter, to_export_main.first, to_export_main.second, (double)it->elapsed_time, machine.time); fclose(f); } mm.unlock(); }
                    if (last_exported_main[i] != to_export_main) {'
    '';
  })
