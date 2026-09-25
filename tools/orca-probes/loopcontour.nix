let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/LayerRegion.cpp \
        --replace-fail '        g.process_classic();
}' '        g.process_classic();
        if (getenv("ORCA_DUMP_LOOP") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_LOOP"), "a"); if (f) { fprintf(f, "LOOP lid=%d n=%zu\n", (int)this->layer()->id(), this->perimeters.entities.size()); for (auto* e : this->perimeters.entities) { Polylines pls = e->as_polylines(); for (auto& pl : pls) { fprintf(f, "L[ "); for (auto& q : pl.points) fprintf(f, "%ld,%ld;", (long)q.x(), (long)q.y()); fprintf(f, "] "); } } fprintf(f, "\n"); fclose(f); } }
}'
    '';
  })
