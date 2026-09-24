let
  pkgs = import <nixpkgs> {};
in
pkgs.orca-slicer.overrideAttrs (old: {
  postPatch = (old.postPatch or "") + ''
    substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
      --replace 'g1_times_cache.push_back({ block.g1_line_id, block.remaining_internal_g1_lines, float(time) });' \
        'g1_times_cache.push_back({ block.g1_line_id, block.remaining_internal_g1_lines, float(time) }); if (const char* p = getenv("ORCA_DUMP_ELAPSED")) { FILE* f=fopen(p,"a"); fprintf(f,"%u %u %.9g\n", block.g1_line_id, block.remaining_internal_g1_lines, time); fclose(f);} '
  '';
})
