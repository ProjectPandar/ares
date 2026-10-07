let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    src = builtins.path { path = /home/indexyz/ares/OrcaSlicer; name = "orca-src"; filter = path: type: builtins.baseNameOf path != ".git"; };
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/ExtrusionProcessor.hpp \
        --replace-fail '            processed_points.push_back({Point3(scaled(curr.position)), extrusion_speed, overlap});' '            processed_points.push_back({Point3(scaled(curr.position)), extrusion_speed, overlap});
            if (getenv("ORCA_DUMP_SPEEDIN") != nullptr) { static std::mutex sb; sb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SPEEDIN"), "a"); if (f) { fprintf(f, "SPD role=%d pos=%.6f,%.6f curr=%.6f next=%.6f art=%.6f w=%.4f h=%.4f spd=%.4f segs=%zu\n", (int)path.role(), curr.position(0), curr.position(1), curr.distance, next.distance, artificial_distance_to_curled_lines, path.width, path.height, extrusion_speed, path.polyline.points.size()); fclose(f); } sb.unlock(); }' \
        --replace-fail '        return processed_points;' '        if (getenv("ORCA_DUMP_SPEEDIN") != nullptr) { static std::mutex eb; eb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SPEEDIN"), "a"); if (f) { fprintf(f, "SPD_END role=%d n=%zu\n", (int)path.role(), processed_points.size()); fclose(f); } eb.unlock(); }
        return processed_points;'
    '';
  })
