let
  pkgs = import <nixpkgs> {};
in
pkgs.orca-slicer.overrideAttrs (old: {
  postPatch = (old.postPatch or "") + ''
    substituteInPlace src/libslic3r/GCode/GCodeProcessor.cpp \
      --replace 'get_time_dhms(machine.time).c_str());' \
        'get_time_dhms(machine.time).c_str()); if (const char* p = getenv("ORCA_TOTAL")) { FILE* f=fopen(p,"a"); fprintf(f,"MACH time=%.9g prep=%.9g\n",machine.time,machine.prepare_time); fclose(f);}'
  '';
})
