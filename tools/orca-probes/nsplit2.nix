let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/Fill.cpp \
        --replace-fail '    // Merge very small areas that is smaller than a single line width to the normal infill if they touches' '    if (getenv("ORCA_DUMP_NSPLIT") != nullptr) { static std::mutex nb; nb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_NSPLIT"), "a"); if (f) { for (const ExPolygon &ex : normal_fill_areas_ex) { fprintf(f, "ONORM n=%d", (int)ex.contour.points.size()); for (const Point &p : ex.contour.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); } fclose(f); } nb.unlock(); }
    // Merge very small areas that is smaller than a single line width to the normal infill if they touches'
    '';
  })
