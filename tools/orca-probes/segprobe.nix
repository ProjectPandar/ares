let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillRectilinear.cpp \
        --replace-fail '	} else {
		traverse_graph_generate_polylines(poly_with_offset, params, segs, this->has_consistent_pattern(), polylines_out);
    }' '	} else {
		if (getenv("ORCA_DUMP_SEGS") != nullptr) { static std::mutex sb; sb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SEGS"), "a"); if (f) { for (const SegmentedIntersectionLine &vl : segs) { fprintf(f, "SEGS z=%.4f x=%ld ", this->z, (long)vl.pos); for (const SegmentIntersection &si : vl.intersections) fprintf(f, "[%d y=%ld prev=%ld,%d next=%ld,%d]", (int)si.type, (long)si.pos(), (long)si.prev_on_contour, (int)si.prev_on_contour_type, (long)si.next_on_contour, (int)si.next_on_contour_type); fprintf(f, "\n"); } fclose(f); } sb.unlock(); }
		traverse_graph_generate_polylines(poly_with_offset, params, segs, this->has_consistent_pattern(), polylines_out);
    }'
    '';
  })
