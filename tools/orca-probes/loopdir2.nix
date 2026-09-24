let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail '    // clip the path to avoid the extruder to get exactly on the first point of the loop' '    if (getenv("ORCA_DUMP_LOOPDIR") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_LOOPDIR"), "a"); if (f) { const Polygon pg = loop.polygon(); double area = 0.; Point prev = pg.points.back(); for (const Point& p : pg.points) { area += (double)(prev.x() + p.x()) * (double)(prev.y() - p.y()); prev = p; } const Point3& p0 = loop.paths.front().polyline.points.front(); const Point3& p1 = loop.paths.front().polyline.points[1]; const Point3& p2 = loop.paths.front().polyline.points[2]; fprintf(f, "AFTER_SEAM layer=%zu role=%d area=%.0f ccw=%d p0=(%lld,%lld) p1=(%lld,%lld) p2=(%lld,%lld)\\n", layer_id(), (int)loop.role(), area, (area < 0.) ? 1 : 0, (long long)p0.x(), (long long)p0.y(), (long long)p1.x(), (long long)p1.y(), (long long)p2.x(), (long long)p2.y()); fclose(f); } }
    // clip the path to avoid the extruder to get exactly on the first point of the loop'
    '';
  })
