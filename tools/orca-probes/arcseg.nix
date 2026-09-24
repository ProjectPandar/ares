let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail 'process_G1(g1_axes, g1_feedrate, G1DiscretizationOrigin::G2G3, remaining_internal_g1_lines);' 'if (getenv("ORCA_DUMP_ARC") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_ARC"), "a"); if (f) { fprintf(f, "SEG %g %g %g %g\n", g1_axes[0].value_or(-999), g1_axes[1].value_or(-999), g1_axes[2].value_or(-999), g1_axes[3].value_or(-999)); fclose(f); } }
          process_G1(g1_axes, g1_feedrate, G1DiscretizationOrigin::G2G3, remaining_internal_g1_lines);'
    '';
  })
