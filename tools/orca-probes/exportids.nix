let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
        --replace-fail 'process_line_move(g1_lines_counter++);' 'if (getenv("ORCA_DUMP_EXPORTIDS") != nullptr) { static std::mutex me; me.lock(); FILE* f = fopen(getenv("ORCA_DUMP_EXPORTIDS"), "a"); if (f) { fprintf(f, "%u %s\\n", g1_lines_counter, gcode_line.c_str()); fclose(f); } me.unlock(); }
                            process_line_move(g1_lines_counter++);'
    '';
  })
