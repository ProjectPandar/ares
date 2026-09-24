let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillRectilinear.cpp \
        --replace-fail '    bounding_box.merge(Slic3r::align_to_grid(bounding_box.min, Point(line_spacing, line_spacing), refpt));
' '    if (getenv("ORCA_DUMP_MFL")!=nullptr){ static std::mutex mm; mm.lock(); FILE*f=fopen(getenv("ORCA_DUMP_MFL"),"a"); if(f){ fprintf(f,"MFL angle=%f ls=%ld ps=%ld refx=%ld refy=%ld bbmin=%ld bbmax=%ld xm=%ld\n",(double)angle,(long)line_spacing,(long)pattern_shift,(long)refpt.x(),(long)refpt.y(),(long)bounding_box.min.x(),(long)bounding_box.max.x(),(long)x_margin); fclose(f);} mm.unlock(); }
    bounding_box.merge(Slic3r::align_to_grid(bounding_box.min, Point(line_spacing, line_spacing), refpt));
'
    '';
  })
