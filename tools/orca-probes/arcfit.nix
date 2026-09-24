let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/ArcFitter.cpp \
        --replace-fail '    result.reserve(points.size() / 2);  //worst case size' '    if (getenv("ORCA_DUMP_ARCFIT") != nullptr) { static std::mutex ab; ab.lock(); FILE* f = fopen(getenv("ORCA_DUMP_ARCFIT"), "a"); if (f) { fprintf(f, "ARC n=%zu tol=%.1f\n", points.size(), tolerance); for (const Point &p : points) fprintf(f, "AP %ld,%ld\n", (long)p.x(), (long)p.y()); fclose(f); } ab.unlock(); }
    result.reserve(points.size() / 2);  //worst case size' \
        --replace-fail '	//BBS: handle the remain data' '    if (getenv("ORCA_DUMP_ARCFIT") != nullptr) { static std::mutex ab3; ab3.lock(); FILE* f = fopen(getenv("ORCA_DUMP_ARCFIT"), "a"); if (f) { for (const PathFittingData &d : result) fprintf(f, "RANGE [%zu,%zu] type=%d\n", d.start_point_index, d.end_point_index, (int)d.path_type); fclose(f); } ab3.unlock(); }
	//BBS: handle the remain data'
    '';
  })
