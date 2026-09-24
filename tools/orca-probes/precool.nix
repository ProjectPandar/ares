let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/CoolingBuffer.cpp \
        --replace-fail '    out = this->apply_layer_cooldown(m_gcode, layer_id, layer_time_stretched, per_extruder_adjustments);' '    if (getenv("ORCA_DUMP_PRECOOL") != nullptr) { static std::mutex pc; pc.lock(); FILE* f = fopen(getenv("ORCA_DUMP_PRECOOL"), "a"); if (f) { fprintf(f, "=== LAYER %zu ===\n", layer_id); fwrite(m_gcode.data(), 1, m_gcode.size(), f); fclose(f); } pc.unlock(); }
    out = this->apply_layer_cooldown(m_gcode, layer_id, layer_time_stretched, per_extruder_adjustments);'
    '';
  })
