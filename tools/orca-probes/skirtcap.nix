let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail 'double ref_speed = speed;' 'if (getenv("ORCA_DUMP_CAP") != nullptr && path.role() == erSkirt) { static std::mutex cq; cq.lock(); FILE* f = fopen(getenv("ORCA_DUMP_CAP"), "a"); if (f) { fprintf(f, "SKIRTCAP post=%.12g mm3=%.12g w=%.9g h=%.9g layer=%d\n", speed, _mm3_per_mm, path.width, path.height, _layer); fclose(f); } cq.unlock(); }
    double ref_speed = speed;'
    '';
  })
