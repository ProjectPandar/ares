let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillConcentricInternal.cpp \
        --replace-fail '        Arachne::WallToolPaths wallToolPaths(polygons, min_spacing, min_spacing, loops_count, 0, params.layer_height, input_params);' '        if (getenv("ORCA_DUMP_CI2") != nullptr) { static std::mutex cb; cb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_CI2"), "a"); if (f) { fprintf(f, "DOM n=%d", (int)expolygon.contour.points.size()); for (const Point &p : expolygon.contour.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, " holes=%zu\n", expolygon.holes.size()); fclose(f); } cb.unlock(); }
        Arachne::WallToolPaths wallToolPaths(polygons, min_spacing, min_spacing, loops_count, 0, params.layer_height, input_params);'
    '';
  })
