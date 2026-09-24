let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Arachne/SkeletalTrapezoidation.cpp \
        --replace-fail 'for (const VD::cell_type &cell : voronoi_diagram.cells()) {' 'for (const VD::cell_type &cell : voronoi_diagram.cells()) { if (getenv("ORCA_DUMP_CELLS") != nullptr) { fprintf(stderr, "CELL id=%lld pt=%d src=%lld\n", (long long)(&cell - voronoi_diagram.cells().data()), (int)cell.contains_point(), (long long)cell.source_index()); }'
    '';
  })
