# Provenance: SupportSpotsGenerator.cpp::estimate_malformations, exact source
# of the case's seed-replay producer (72d4hac8qmg5p9m22v2kifbp1hkxjk63.drv).
# Records the selected predecessor
# and every indexed candidate at the layer-2 degenerate corner. Before use,
# compare plate_1.gcode to case-6qvP0t/orca.gcode, normalizing only generated-by.
let
  pkgs = import <nixpkgs> {};
in
pkgs.orca-slicer.overrideAttrs (old: {
  src = /nix/store/g3wj93lyhjsvc32fmgvw0cdbd6191q55-orca-slicer-2.4.2-seed-replay-source;
  postPatch = (old.postPatch or "") + ''
    substituteInPlace src/libslic3r/Support/SupportSpotsGenerator.cpp \
      --replace-fail '// correctify the distance sign using slice polygons' 'if (l->id() == 2 && i == 0 && getenv("ORCA_DUMP_BOTTOMLINE") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_BOTTOMLINE"), "a"); if (f) { fprintf(f, "SELECT layer=%zu point=%zu mid=%.9f,%.9f selected=%zu distance=%.9g height=%.9f count=%zu\n", l->id(), i, middle.x(), middle.y(), bottom_line_idx, double(middle_distance), bottom_line.curled_up_height, prev_layer_lines.get_lines().size()); size_t idx = 0; for (const auto& candidate : prev_layer_lines.get_lines()) { fprintf(f, "CAND index=%zu a=%.9f,%.9f b=%.9f,%.9f squared=%.9g height=%.9f\n", idx++, candidate.a.x(), candidate.a.y(), candidate.b.x(), candidate.b.y(), line_alg::distance_to_squared(candidate, middle), candidate.curled_up_height); } fclose(f); } }
                    // correctify the distance sign using slice polygons'
  '';
})
