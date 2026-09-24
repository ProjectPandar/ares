let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillPlanePath.cpp \
        --replace-fail '        polylines = intersection_pl(std::move(polylines), expolygon);' '        if (getenv("ORCA_DUMP_PRECLIP") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_PRECLIP"), "a"); if (f) { for (const Polyline &pl : polylines) { fprintf(f, "IN n=%d", (int)pl.points.size()); for (const Point &p : pl.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); } fclose(f); } }
        polylines = intersection_pl(std::move(polylines), expolygon);
        if (getenv("ORCA_DUMP_PRECLIP") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_PRECLIP"), "a"); if (f) { for (const Polyline &pl : polylines) { fprintf(f, "OUT n=%d", (int)pl.points.size()); for (const Point &p : pl.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); } fclose(f); } }'
    '';
  })
