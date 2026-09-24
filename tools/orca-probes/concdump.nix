let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillConcentricInternal.cpp \
        --replace-fail '            thick_polylines_out.emplace_back(std::move(thick_polyline));' '            if (getenv("ORCA_DUMP_CONC") != nullptr) { static std::mutex m3; m3.lock(); FILE* f3 = fopen(getenv("ORCA_DUMP_CONC"), "a"); if (f3) { for (size_t wi = 0; wi < thick_polyline.width.size(); ++wi) fprintf(f3, ",%lld", (long long)thick_polyline.width[wi]); fprintf(f3, "\n"); fclose(f3);} m3.unlock(); }
            thick_polylines_out.emplace_back(std::move(thick_polyline));'
    '';
  })
