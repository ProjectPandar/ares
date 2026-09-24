let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail '[speed](const ProcessedPoint &p) { return fabs(double(p.speed) - speed) > 1; }); // Ignore small speed variations (under 1mm/sec)' '[speed](const ProcessedPoint &p) { return fabs(double(p.speed) - speed) > 1; }); // Ignore small speed variations (under 1mm/sec)
            if (getenv("ORCA_DUMP_OHQ") != nullptr) { static std::mutex hb; hb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_OHQ"), "a"); if (f) { fprintf(f, "OHQ layer=%d role=%d speed=%.3f var=%d n=%zu\n", m_layer_index, (int)path.role(), speed, (int)variable_speed, new_points.size()); for (const ProcessedPoint &p : new_points) fprintf(f, "  P %ld,%ld speed=%.4f overlap=%.3f\n", (long)p.p.x(), (long)p.p.y(), p.speed, p.overlap); fclose(f); } hb.unlock(); }'
    '';
  })
