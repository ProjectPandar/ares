let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCodeWriter.cpp \
        --replace-fail '    if (m_lifted == 0 && m_to_lift == 0 && target_lift > 0) {' '    if (getenv("ORCA_DUMP_LIFT") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_LIFT"), "a"); if (f) { fprintf(f, "LAZY posz=%.6f lifted=%.6f to_lift=%.6f target=%.6f type=%d\\n", m_pos(2), m_lifted, m_to_lift, target_lift, (int)lift_type); fclose(f); } }
    if (m_lifted == 0 && m_to_lift == 0 && target_lift > 0) {'
    '';
  })
