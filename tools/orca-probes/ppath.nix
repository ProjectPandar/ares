let pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail 'if (!m_config.enable_arc_fitting || path.polyline.fitting_result.empty() || m_config.spiral_mode || sloped != nullptr || path.z_contoured) {' 'if (getenv("ORCA_DUMP_PPATH") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_PPATH"), "a"); if (f) { fprintf(f, "PPATH %u ", unsigned(path.role())); for (const Vec3crd& p : path.polyline.points) fprintf(f, "%ld,%ld ", (long)p(0), (long)p(1)); fprintf(f, "\n"); fclose(f); } }
        if (!m_config.enable_arc_fitting || path.polyline.fitting_result.empty() || m_config.spiral_mode || sloped != nullptr || path.z_contoured) {'
    '';
  })
