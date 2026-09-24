let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail '[speed](const ProcessedPoint &p) { return fabs(double(p.speed) - speed) > 1; }); // Ignore small speed variations (under 1mm/sec)' '[speed](const ProcessedPoint &p) { return fabs(double(p.speed) - speed) > 1; }); // Ignore small speed variations (under 1mm/sec)
            if (getenv("ORCA_DUMP_OHQ") != nullptr) { static std::mutex hb; hb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_OHQ"), "a"); if (f) { fprintf(f, "OHQ layer=%d role=%d speed=%.3f var=%d n=%zu\n", m_layer_index, (int)path.role(), speed, (int)variable_speed, new_points.size()); } hb.unlock(); }'
      substituteInPlace src/libslic3r/GCode/ExtrusionProcessor.hpp \
        --replace-fail '        next_point.distance = distance + boundary_offset;' '        next_point.distance = distance + boundary_offset;
        if (getenv("ORCA_DUMP_EPP") != nullptr) { static std::mutex eb; eb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_EPP"), "a"); if (f) { fprintf(f, "EPP %.6f,%.6f raw=%.6f ext=%.6f lines=%zu\n", next_point.position.x(), next_point.position.y(), distance, next_point.distance, unscaled_prev_layer.get_lines().size()); fclose(f); } eb.unlock(); }'
    '';
  })
