let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail "m_feedrate = (*feedrate) * MMMIN_TO_MMSEC;" "m_feedrate = (*feedrate) * MMMIN_TO_MMSEC; if (getenv(\"ORCA_DUMP_F3\") != nullptr) { FILE* f3 = fopen(getenv(\"ORCA_DUMP_F3\"), \"a\"); if (f3) { fprintf(f3, \"linef=%.6f m_feedrate=%.9f\n\", (*feedrate), m_feedrate); fclose(f3); } }"
    '';
  })
