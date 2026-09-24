let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Arachne/SkeletalTrapezoidation.cpp \
        --replace-fail 'for (const VD::cell_type &cell : voronoi_diagram.cells()) {' 'for (const VD::cell_type &cell : voronoi_diagram.cells()) { if (getenv("ORCA_DUMP_CELLS") != nullptr) { const auto* se0 = cell.incident_edge(); long long vx = -2000000000, vy = -2000000000; if (se0 && se0->vertex1()) { vx = (long long)se0->vertex1()->x(); vy = (long long)se0->vertex1()->y(); } fprintf(stderr, "CELL id=%lld pt=%d src=%lld v=(%lld,%lld)\n", (long long)(&cell - voronoi_diagram.cells().data()), (int)cell.contains_point(), (long long)cell.source_index(), vx, vy); }'
    '';
  })
