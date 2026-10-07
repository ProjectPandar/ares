let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    src = builtins.path { path = /home/indexyz/ares/OrcaSlicer; name = "orca-src"; filter = path: type: builtins.baseNameOf path != ".git"; };
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail '        double last_set_speed = new_points[0].speed * 60.0;' '            double last_set_speed = new_points[0].speed * 60.0;
            if (getenv("ORCA_DUMP_PATHSPD") != nullptr) { static std::mutex pb; pb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_PATHSPD"), "a"); if (f) { fprintf(f, "PATH role=%d npts=%zu F=%.4f v0=%.4f vlast=%.4f\n", (int)path.role(), new_points.size(), F, new_points[0].speed*60.0, new_points[new_points.size()-2].speed*60.0); fclose(f); } pb.unlock(); }' \
        --replace-fail 'gcode += gcodegen.writer().set_speed(_wipe_speed * 60, "", cooling_mark);' 'if (getenv("ORCA_DUMP_PATHSPD") != nullptr) { static std::mutex wb; wb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_PATHSPD"), "a"); if (f) { fprintf(f, "WIPE cur=%.4f chosen=%.4f\n", gcodegen.writer().get_current_speed(), _wipe_speed * 60); fclose(f); } wb.unlock(); }
                gcode += gcodegen.writer().set_speed(_wipe_speed * 60, "", cooling_mark);'
    '';
  })
