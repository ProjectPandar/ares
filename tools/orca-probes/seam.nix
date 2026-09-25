let pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/ExtrusionEntity.cpp \
        --replace-fail '    auto [path_idx, segment_idx, p] = get_closest_path_and_point(point, prefer_non_overhang);' '    if (getenv("ORCA_DUMP_SEAM") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_SEAM"), "a"); if (f) { fprintf(f, "SEAMIN %ld,%ld ", (long)point.x(), (long)point.y()); fclose(f); } }
    auto [path_idx, segment_idx, p] = get_closest_path_and_point(point, prefer_non_overhang);
    if (getenv("ORCA_DUMP_SEAM") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_SEAM"), "a"); if (f) { fprintf(f, "FOOT %ld,%ld seg%d\n", (long)p.x(), (long)p.y(), int(segment_idx)); fclose(f); } }'
      substituteInPlace src/libslic3r/ExtrusionEntity.cpp \
        --replace-fail '    // split at the new vertex
    this->split_at_vertex(p);
}' '    // split at the new vertex
    this->split_at_vertex(p);
    if (getenv("ORCA_DUMP_SEAM") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_SEAM"), "a"); if (f) { for (auto& pp : this->paths.front().polyline.points) fprintf(f, "S %ld,%ld;", (long)pp.x(), (long)pp.y()); fprintf(f, "\n"); fclose(f); } }
}'
    '';
  })
