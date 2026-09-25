let pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/SeamPlacer.cpp \
        --replace-fail 'overhang = layer_perimeters.points[seam_index].unsupported_dist;' '{
    const Perimeter& perimeter2 = layer_perimeters.points[seam_index].perimeter;
    if (getenv("ORCA_DUMP_SCORE") != nullptr) {
      FILE* f = fopen(getenv("ORCA_DUMP_SCORE"), "a");
      if (f) {
        fprintf(f, "WIN z=%.3f idx=%zu pos %.6f,%.6f last %.3f,%.3f range %zu..%zu\n",
          unscaled_z, seam_index,
          layer_perimeters.points[seam_index].position.x(),
          layer_perimeters.points[seam_index].position.y(),
          unscaled<float>(last_pos).x(), unscaled<float>(last_pos).y(),
          perimeter2.start_index, perimeter2.end_index);
        for (size_t ci = perimeter2.start_index; ci < perimeter2.end_index; ++ci) {
          const auto& c = layer_perimeters.points[ci];
          fprintf(f, "CAND %zu %.6f,%.6f oh=%.4f vis=%.4f emb=%.3f ang=%.4f t=%d fw=%.4f\n",
            ci, c.position.x(), c.position.y(), c.overhang, c.visibility,
            c.embedded_distance, c.local_ccw_angle, int(c.type), perimeter2.flow_width);
        }
        fclose(f);
      }
    }
  }
  overhang = layer_perimeters.points[seam_index].unsupported_dist;'
    '';
  })
