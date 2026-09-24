let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode.cpp \
        --replace-fail 'return fan_mover->process_gcode(in, true);' 'if (getenv("ORCA_DUMP_FANIN") != nullptr) { static std::mutex fi; fi.lock(); FILE* f = fopen(getenv("ORCA_DUMP_FANIN"), "a"); if (f) { fprintf(f, "=== CHUNK ===\\n"); fwrite(in.data(), 1, in.size(), f); fclose(f); } fi.unlock(); }
            return fan_mover->process_gcode(in, true);'
    '';
  })
