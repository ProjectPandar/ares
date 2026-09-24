let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/LayerRegion.cpp \
        --replace-fail '        path->simplify(scaled_resolution);' '        if (getenv("ORCA_DUMP_SIMP") != nullptr && path->role() == erBottomSurface) { FILE* f = fopen(getenv("ORCA_DUMP_SIMP"), "a"); if (f) { fprintf(f, "PRE n=%d", (int)path->polyline.points.size()); for (const Point3 &p : path->polyline.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); fclose(f); } }
        path->simplify(scaled_resolution);
        if (getenv("ORCA_DUMP_SIMP") != nullptr && path->role() == erBottomSurface) { FILE* f = fopen(getenv("ORCA_DUMP_SIMP"), "a"); if (f) { fprintf(f, "POST n=%d", (int)path->polyline.points.size()); for (const Point3 &p : path->polyline.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); fclose(f); } }'
    '';
  })
