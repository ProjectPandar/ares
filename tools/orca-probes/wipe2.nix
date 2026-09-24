let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail '                    double dE = length * (segment_length / wipe_dist);' '                    double dE = length * (segment_length / wipe_dist);
                    if (getenv("ORCA_DUMP_WIPE2") != nullptr) { static std::mutex wb2; wb2.lock(); FILE* f = fopen(getenv("ORCA_DUMP_WIPE2"), "a"); if (f) { fprintf(f, "WSEG len=%.17g dE=%.17g length=%.17g wipe_dist=%.17g\n", segment_length, dE, length, wipe_dist); fclose(f); } wb2.unlock(); }' \
        --replace-fail '                gcode += gcodegen.writer().set_speed(_wipe_speed * 60, "", cooling_mark);' '                if (getenv("ORCA_DUMP_WIPE2") != nullptr) { static std::mutex wb; wb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_WIPE2"), "a"); if (f) { fprintf(f, "WIPE2 pathlen=%.17g wipe_dist_cfg=%.17g npts=%zu\n", wipe_path.length(), wipe_dist, wipe_path.points.size()); for (const Point &p : wipe_path.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); fclose(f); } wb.unlock(); }
                gcode += gcodegen.writer().set_speed(_wipe_speed * 60, "", cooling_mark);'
    '';
  })
