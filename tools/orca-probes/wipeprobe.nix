let
  pkgs = import <nixpkgs> {};
in
pkgs.orca-slicer.overrideAttrs (old: {
  postPatch = (old.postPatch or "") + ''
    substituteInPlace src/libslic3r/GCode.cpp \
      --replace 'double dE = length * (segment_length / wipe_dist);' \
        'double dE = length * (segment_length / wipe_dist); if (const char* p = getenv("ORCA_WIPE_SEG")) { FILE* f=fopen(p,"a"); fprintf(f,"OWIPE len=%.17g dist=%.17g\n", length, wipe_dist); fprintf(f,"OSEG slen=%.17g dE=%.17g\n", segment_length, dE); fclose(f);} '
  '';
})
