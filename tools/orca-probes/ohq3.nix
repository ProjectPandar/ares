let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail '[speed](const ProcessedPoint &p) { return fabs(double(p.speed) - speed) > 1; }); // Ignore small speed variations (under 1mm/sec)' '[speed](const ProcessedPoint &p) { return fabs(double(p.speed) - speed) > 1; }); // Ignore small speed variations (under 1mm/sec)
            if (getenv("ORCA_DUMP_OHQ") != nullptr) { static std::mutex hb; hb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_OHQ"), "a"); if (f) { fprintf(f, "OHQ layer=%d role=%d speed=%.3f var=%d n=%zu\n", m_layer_index, (int)path.role(), speed, (int)variable_speed, new_points.size()); for (const ProcessedPoint &p : new_points) fprintf(f, "  P %ld,%ld speed=%.4f overlap=%.6f dist=%.6f\n", (long)p.p.x(), (long)p.p.y(), p.speed, p.overlap, (1.0f - p.overlap) * path.width); fclose(f); } hb.unlock(); }'
      substituteInPlace src/libslic3r/GCode/ExtrusionProcessor.hpp \
        --replace-fail '        prev_layer_boundaries[object] = next_layer_boundaries[object];' '        if (getenv("ORCA_DUMP_LSLICES") != nullptr) { static std::mutex lb; lb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_LSLICES"), "a"); if (f) { fprintf(f, "LSLICES layer=%zu polys=%zu\n", layer->id(), layer->lslices.size()); for (const ExPolygon &ex : layer->lslices) { fprintf(f, " POLY n=%zu", ex.contour.points.size()); for (const Point &p : ex.contour.points) fprintf(f, " %ld,%ld", (long)p.x(), (long)p.y()); fprintf(f, "\n"); } fclose(f); } lb.unlock(); }
        prev_layer_boundaries[object] = next_layer_boundaries[object];'
    '';
  })
