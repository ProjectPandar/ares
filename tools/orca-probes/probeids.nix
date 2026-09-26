let
  pkgs = import <nixpkgs> {};
in
pkgs.orca-slicer.overrideAttrs (old: {
  postPatch = (old.postPatch or "") + ''
    substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
      --replace 'process_line_move(g1_lines_counter++);' \
        'if (const char* pp = getenv("ORCA_DUMP_PROBEIDS")) { FILE* f=fopen(pp,"a"); fprintf(f,"PROBE %zu %s", g1_lines_counter, gcode_line.c_str()); fclose(f);} process_line_move(g1_lines_counter++);'
  '';
})
