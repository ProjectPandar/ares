let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Arachne/WallToolPaths.cpp \
        --replace-fail '    // Removing collinear edges may introduce self intersections, so we need to fix them again' '    if (getenv("ORCA_DUMP_PREP") != nullptr) { static std::mutex pb; pb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_PREP"), "a"); if (f) { fprintf(f, "PREP n=%zu", prepared_outline.size()); for (const Polygon &p : prepared_outline) { fprintf(f, " |%zu", p.points.size()); for (const Point &q : p.points) fprintf(f, " %ld,%ld", (long)q.x(), (long)q.y()); } fprintf(f, "\n"); fclose(f); } pb.unlock(); }
    // Removing collinear edges may introduce self intersections, so we need to fix them again'
    '';
  })
