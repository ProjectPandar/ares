let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Layer.cpp \
        --replace-fail '    // populate slices vector' 'if (getenv("ORCA_DUMP_LSLICES") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_LSLICES"), "a"); if (f) { fprintf(f, "LSLICES n=%zu pts=", slices.size()); for (auto& ex : slices) { fprintf(f, "("); for (auto& p : ex.contour.points) fprintf(f, "%lld.%lld,", (long long)p.x(), (long long)p.y()); fprintf(f, ") "); } fprintf(f, "\n"); fclose(f); } }
    // populate slices vector'
    '';
  })
