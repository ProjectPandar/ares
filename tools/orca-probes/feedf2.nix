let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail 'if (line.has_f())
        m_feedrate = line.f() * MMMIN_TO_MMSEC;' 'if (line.has_f()) { m_feedrate = line.f() * MMMIN_TO_MMSEC; if (getenv("ORCA_DUMP_F2") != nullptr) { FILE* f2 = fopen(getenv("ORCA_DUMP_F2"), "a"); if (f2) { fprintf(f2, "linef=%.6f m_feedrate=%.9f\\n", line.f(), m_feedrate); fclose(f2); } } }'
    '';
  })
