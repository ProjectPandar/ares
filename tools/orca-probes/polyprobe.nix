let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillRectilinear.cpp \
        --replace-fail '    for (Polylines::iterator it = polylines_out.begin() + n_polylines_out_initial; it != polylines_out.end(); ++ it) {' '    if (getenv("ORCA_DUMP_POLY") != nullptr) { static std::mutex pb; pb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_POLY"), "a"); if (f) { for (size_t pi = n_polylines_out_initial; pi < polylines_out.size(); ++ pi) { fprintf(f, "POLY z=%.4f ", this->z); for (const Point &p : polylines_out[pi].points) fprintf(f, "%ld %ld ", (long)p.x(), (long)p.y()); fprintf(f, "\n"); } fclose(f); } pb.unlock(); }
    for (Polylines::iterator it = polylines_out.begin() + n_polylines_out_initial; it != polylines_out.end(); ++ it) {'
    '';
  })
