let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillRectilinear.cpp \
        --replace-fail '	coord_t x0 = bounding_box.min(0);' '	coord_t x0 = bounding_box.min(0);
        if (getenv("ORCA_DUMP_GRID")!=nullptr){ static std::mutex mg; mg.lock(); FILE*f=fopen(getenv("ORCA_DUMP_GRID"),"a"); if(f){ fprintf(f,"GRID pat=%d dens=%f spacing=%f ls=%ld x0=%ld bbmin=%ld bbmax=%ld nv=%zu\n",(int)params.pattern,(double)params.density,(double)this->spacing,(long)line_spacing,(long)x0,(long)bounding_box.min(0),(long)bounding_box.max(0),(size_t)n_vlines); fclose(f);} mg.unlock(); }'
    '';
  })
