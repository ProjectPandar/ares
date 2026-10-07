let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    src = builtins.path { path = /home/indexyz/ares/OrcaSlicer; name = "orca-src"; filter = path: type: builtins.baseNameOf path != ".git"; };
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail 'auto dE = e_per_mm * line_length;' 'auto dE = e_per_mm * line_length;
                if (getenv("ORCA_DUMP_EMIT") != nullptr) { static std::mutex xb; xb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_EMIT"), "a"); if (f) { fprintf(f, "EMITC role=%d ll=%.6f dE=%.8f e_per_mm=%.8f mm3=%.8f F=%.4f\n", (int)path.role(), line_length, dE, e_per_mm, _mm3_per_mm, F); fclose(f); } xb.unlock(); }' \
        --replace-fail 'double new_speed = pre_processed_point.speed * 60.0;' 'double new_speed = pre_processed_point.speed * 60.0;
            if (getenv("ORCA_DUMP_EMIT") != nullptr) { static std::mutex vb; vb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_EMIT"), "a"); if (f) { fprintf(f, "EMITV role=%d i=%zu ll=%.6f pl=%.6f tl=%.6f dE=%.8f e_per_mm=%.8f mm3=%.8f F=%.4f ns=%.4f last=%.4f sloped=%d\n", (int)path.role(), i, line_length, path_length, total_length, e_per_mm * line_length, e_per_mm, _mm3_per_mm, F, new_speed, last_set_speed, sloped != nullptr); fclose(f); } vb.unlock(); }'
    '';
  })
