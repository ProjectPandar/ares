let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/Fill.cpp \
        --replace-fail '            split_solid_surface(layer.id(), surface_fills[i], normal_infill, narrow_infill);
' '            split_solid_surface(layer.id(), surface_fills[i], normal_infill, narrow_infill);
            if (getenv("ORCA_DUMP_NSPLIT") != nullptr) { static std::mutex nb; nb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_NSPLIT"), "a"); if (f) { fprintf(f, "NSP layer=%zu normal=%d narrow=%d\n", layer.id(), (int)normal_infill.size(), (int)narrow_infill.size()); for (const ExPolygon &ex : narrow_infill) { fprintf(f, "NSN n=%d", (int)ex.contour.points.size()); for (const Point &p : ex.contour.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); } fclose(f); } nb.unlock(); }
'
    '';
  })
