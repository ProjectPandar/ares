let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/Fill.cpp \
        --replace-fail 'void split_solid_surface(size_t layer_id, const SurfaceFill &fill, ExPolygons &normal_infill, ExPolygons &narrow_infill)
{
    assert(fill.surface.surface_type == stInternalSolid);' 'void split_solid_surface(size_t layer_id, const SurfaceFill &fill, ExPolygons &normal_infill, ExPolygons &narrow_infill)
{
    assert(fill.surface.surface_type == stInternalSolid);
    if (getenv("ORCA_DUMP_FILL") != nullptr) { static std::mutex fb; fb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_FILL"), "a"); if (f) { fprintf(f, "FILL n=%zu\n", fill.expolygons.size()); for (const ExPolygon &ex : fill.expolygons) { fprintf(f, "FEXP n=%d", (int)ex.contour.points.size()); for (const Point &p : ex.contour.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); for (const Polygon &h : ex.holes) { fprintf(f, "FHOLE n=%d", (int)h.points.size()); for (const Point &p : h.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); } } fclose(f); } fb.unlock(); }'
    '';
  })
