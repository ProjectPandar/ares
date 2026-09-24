let
  pkgs = import <nixpkgs> {};
in
pkgs.orca-slicer.overrideAttrs (old: {
  postPatch = (old.postPatch or "") + ''
    substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
      --replace 'std::pair<int, int> to_export_main = { int(100.0f * it->elapsed_time / machine.time),' \
        'if (const char* p = getenv("ORCA_M73TRACE")) { FILE* f=fopen(p,"a"); fprintf(f,"ctr=%zu id=%u el=%.9g mt=%.9g\n", g1_lines_counter, it->id, (double)it->elapsed_time, machine.time); fclose(f);} std::pair<int, int> to_export_main = { int(100.0f * it->elapsed_time / machine.time),'
  '';
})
