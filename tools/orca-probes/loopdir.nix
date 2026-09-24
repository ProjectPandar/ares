let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail '    // find the point of the loop that is closest to the current extruder position' '    if (getenv("ORCA_DUMP_LOOPDIR") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_LOOPDIR"), "a"); if (f) { const Polygon pg = loop.polygon(); double area = 0.; Point prev = pg.points.back(); for (const Point& p : pg.points) { area += (double)(prev.x() + p.x()) * (double)(prev.y() - p.y()); prev = p; } fprintf(f, "LOOP layer=%zu role=%d hole=%d area=%.1f ccw=%d np=%zu\\n", layer_id(), (int)loop.role(), (int)is_hole, area, (area < 0.) ? 1 : 0, pg.points.size()); fclose(f); } }
    // find the point of the loop that is closest to the current extruder position'
    '';
  })
