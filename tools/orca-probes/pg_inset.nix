let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/PerimeterGenerator.cpp \
        --replace-fail '            float(min_perimeter_infill_spacing / 2.));' '            float(min_perimeter_infill_spacing / 2.));
        if (getenv("ORCA_DUMP_INSET") != nullptr) { static std::mutex mb; mb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_INSET"), "a"); if (f) { fprintf(f, "PGI lid=%d ln=%d inset=%ld iwo=%ld mpis=%ld\n", (int)this->layer_id, (int)loop_number, (long)inset, (long)infill_peri_overlap, (long)min_perimeter_infill_spacing); for (auto&e:not_filled_exp){ BoundingBox b=e.contour.bounding_box(); fprintf(f,"NF min=%ld %ld max=%ld %ld\n",(long)b.min.x(),(long)b.min.y(),(long)b.max.x(),(long)b.max.y()); } for (auto&e:infill_exp){ BoundingBox b=e.contour.bounding_box(); fprintf(f,"IE min=%ld %ld max=%ld %ld\n",(long)b.min.x(),(long)b.min.y(),(long)b.max.x(),(long)b.max.y()); } fclose(f); } mb.unlock(); }'
    '';
  })
