let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail 'if (travel.size() == 2) {' 'if (getenv("ORCA_DUMP_TV") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_TV"), "a"); if (f) { fprintf(f, "TVEMIT n=%zu", travel.size()); for (const Point& p : travel.points) fprintf(f, " (%lld,%lld)", (long long)p.x(), (long long)p.y()); fprintf(f, "\\n"); fclose(f); } }
            if (travel.size() == 2) {'
    '';
  })
