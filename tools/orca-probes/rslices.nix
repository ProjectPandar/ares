let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/LayerRegion.cpp \
        --replace-fail '    for (Surface &surface : this->slices.surfaces)' 'if (getenv("ORCA_DUMP_RSLICES") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_RSLICES"), "a"); if (f) { fprintf(f, "RSLICES n=%zu pts=", this->slices.surfaces.size()); for (auto& s : this->slices.surfaces) { fprintf(f, "("); for (auto& p : s.expolygon.contour.points) fprintf(f, "%lld.%lld,", (long long)p.x(), (long long)p.y()); fprintf(f, ") "); } fprintf(f, "\n"); fclose(f); } }
    for (Surface &surface : this->slices.surfaces)'
    '';
  })
