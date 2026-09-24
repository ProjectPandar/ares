let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail '    float value_t;' '    if (getenv("ORCA_DUMP_SAFE") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_SAFE"), "a"); if (f) { float vs; fprintf(f, "M221 raw=[%s] hasS=%d hasValueS=%d v=%.3f pct_now=%.4f\\n", line.raw().c_str(), (int)line.has((char)83), (int)line.has_value((char)83, vs), vs, m_time_processor.machines[0].extrude_factor_override_percentage); fclose(f); } }
    float value_t;'
    '';
  })
