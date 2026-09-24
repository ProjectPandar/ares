let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillBase.cpp \
        --replace-fail '    unextruded_areas = diff_ex(this->no_overlap_expolygons, union_ex(out->polygons_covered_by_spacing(10)));' '    if (getenv("ORCA_DUMP_RESID") != nullptr) { size_t dpts = 0; for (const auto& ex : this->no_overlap_expolygons) dpts += ex.contour.points.size(); fprintf(stderr, "RESID domain_n=%zu domain_pts=%zu surf_pts=%zu out_n=%zu\n", this->no_overlap_expolygons.size(), dpts, surface->expolygon.contour.points.size(), out->entities.size()); }
    unextruded_areas = diff_ex(this->no_overlap_expolygons, union_ex(out->polygons_covered_by_spacing(10)));'
    '';
  })
