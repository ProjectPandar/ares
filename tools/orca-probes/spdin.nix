let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/ExtrusionProcessor.hpp \
        --replace-fail '            float extrusion_speed = std::min(calculate_speed(curr.distance), calculate_speed(next.distance));' '            float extrusion_speed = std::min(calculate_speed(curr.distance), calculate_speed(next.distance));
            if (getenv("ORCA_DUMP_SPEEDIN") != nullptr) { static std::mutex sb; sb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SPEEDIN"), "a"); if (f) { fprintf(f, "SPD curr=%.6f next=%.6f art=%.6f e0=%.4f s0=%.4f min1=%.4f s1=%.4f\n", curr.distance, next.distance, artificial_distance_to_curled_lines, speed_sections.front().first, speed_sections.front().second, speed_sections[1].first, speed_sections[1].second); fclose(f); } sb.unlock(); }'
    '';
  })
