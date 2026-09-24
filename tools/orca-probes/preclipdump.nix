let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillConcentricInternal.cpp \
        --replace-fail '        Arachne::WallToolPaths wallToolPaths(polygons, min_spacing, min_spacing, loops_count, 0, params.layer_height, input_params);' '        Arachne::WallToolPaths wallToolPaths(polygons, min_spacing, min_spacing, loops_count, 0, params.layer_height, input_params);
        if (getenv("ORCA_DUMP_PRECLIP") != nullptr) { for (const auto& inset : wallToolPaths.getToolPaths()) { for (const auto& ex : inset) { if (ex.empty()) continue; fprintf(stderr, "PRECLIP n=%zu w0=%lld\n", ex.junctions.size(), (long long)ex.junctions.front().w); } } }'
    '';
  })
