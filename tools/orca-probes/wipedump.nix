let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail '        wipe_path.clip_end(wipe_path.length() - wipe_dist);' '        if (getenv("ORCA_DUMP_WIPE") != nullptr) { static std::mutex mw; mw.lock(); FILE* f = fopen(getenv("ORCA_DUMP_WIPE"), "a"); if (f) { fprintf(f, "WP"); for (const Point& p : wipe_path.points) fprintf(f, " (%lld,%lld)", (long long)p.x(), (long long)p.y()); fprintf(f, " len=%.6f dist=%.6f\\n", wipe_path.length(), wipe_dist); fclose(f); } mw.unlock(); }
        wipe_path.clip_end(wipe_path.length() - wipe_dist);'
    '';
  })
