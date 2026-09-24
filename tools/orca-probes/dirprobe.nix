let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillBase.cpp \
        --replace-fail '    return std::pair<float, Point>(out_angle, out_shift);' '    if (getenv("ORCA_DUMP_DIR") != nullptr) { static std::mutex db; db.lock(); FILE* f = fopen(getenv("ORCA_DUMP_DIR"), "a"); if (f) { fprintf(f, "DIR layer_id=%zu thick=%u dontalt=%d angle=%.5f fixed=%d bridge=%.5f type=%d z=%.4f out=%.5f\n", this->layer_id, (unsigned)surface->thickness_layers, (int)this->dont_alternate_fill_direction, this->angle, (int)this->fixed_angle, surface->bridge_angle, (int)surface->surface_type, this->z, out_angle); fclose(f); } db.unlock(); }
    return std::pair<float, Point>(out_angle, out_shift);'
    '';
  })
