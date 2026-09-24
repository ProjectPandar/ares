let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/PerimeterGenerator.cpp \
        --replace-fail '            ThickPolylines polylines;' '            if (getenv("ORCA_DUMP_PGAP") != nullptr) { static std::mutex gb; gb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_PGAP"), "a"); if (f) { fprintf(f, "PGAP min=%.1f max=%.1f gaps_ex_n=%d simp=%.1f\n", min, max, (int)gaps_ex.size(), surface_simplify_resolution); for (const ExPolygon &ex : gaps_ex) { fprintf(f, "PGEX n=%d", (int)ex.contour.points.size()); for (const Point &p : ex.contour.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); } fclose(f); } gb.unlock(); }
            ThickPolylines polylines;' \
        --replace-fail '            // SoftFever: filter out tiny gap fills' '            if (getenv("ORCA_DUMP_PGAP") != nullptr) { static std::mutex gb2; gb2.lock(); FILE* f = fopen(getenv("ORCA_DUMP_PGAP"), "a"); if (f) { for (const ThickPolyline &p : polylines) { fprintf(f, "PGPL len=%.1f n=%d w0=%.1f wN=%.1f", unscale_(p.length()), (int)p.points.size(), unscale_(p.width.front()), unscale_(p.width.back())); for (const Point &pt : p.points) fprintf(f, " %ld,%ld", (long)pt.x(), (long)pt.y()); fprintf(f, "\n"); } fclose(f); } gb2.unlock(); }
            // SoftFever: filter out tiny gap fills'
    '';
  })
