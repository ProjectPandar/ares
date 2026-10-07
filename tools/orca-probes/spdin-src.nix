let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    src = builtins.path { path = /home/indexyz/ares/OrcaSlicer; name = "orca-src"; filter = path: type: builtins.baseNameOf path != ".git"; };
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/ExtrusionProcessor.hpp \
        --replace-fail '            float extrusion_speed = std::min(calculate_speed(curr.distance), calculate_speed(next.distance));' '            float extrusion_speed = std::min(calculate_speed(curr.distance), calculate_speed(next.distance));
            if (getenv("ORCA_DUMP_SPEEDIN") != nullptr) { static std::mutex sb; sb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SPEEDIN"), "a"); if (f) { fprintf(f, "SPD role=%d curr=%.6f next=%.6f art=%.6f w=%.4f h=%.4f spd=%.4f segs=%zu\n", (int)path.role(), curr.distance, next.distance, artificial_distance_to_curled_lines, path.width, path.height, extrusion_speed, path.polyline.points.size()); fclose(f); } sb.unlock(); }'
    '';
  })
