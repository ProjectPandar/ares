let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillConcentric.cpp \
        --replace-fail '    ExPolygons last { std::move(contracted) };' '    if (getenv("ORCA_DUMP_CONC")!=nullptr){ static std::mutex mc; mc.lock(); FILE*f=fopen(getenv("ORCA_DUMP_CONC"),"a"); if(f){ BoundingBox bc=get_extents(to_polygons(expolygon)); fprintf(f,"CONC lid=%d spacing=%f multiline=%d min_spacing=%ld density=%f distance=%ld bbox=[%ld %ld %ld %ld] contractn=%zu\n",(int)this->layer_id,this->spacing,params.multiline,(long)min_spacing,params.density,(long)distance,(long)bc.min.x(),(long)bc.min.y(),(long)bc.max.x(),(long)bc.max.y(),contracted.size()); if(!contracted.empty()){ BoundingBox bk=get_extents(to_polygons(contracted)); fprintf(f,"CONC contracted=[%ld %ld %ld %ld]\n",(long)bk.min.x(),(long)bk.min.y(),(long)bk.max.x(),(long)bk.max.y()); } fclose(f);} mc.unlock(); }
    ExPolygons last { std::move(contracted) };'
    '';
  })
