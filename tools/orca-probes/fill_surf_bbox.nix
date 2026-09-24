let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/Fill.cpp \
        --replace-fail '	        			fill.expolygons.emplace_back(std::move(fill.surface.expolygon));' '	        			fill.expolygons.emplace_back(std::move(fill.surface.expolygon));
	        			if (getenv("ORCA_DUMP_FS")!=nullptr && surface.surface_type==stInternal){ static std::mutex mb; mb.lock(); FILE*f=fopen(getenv("ORCA_DUMP_FS"),"a"); if(f){ BoundingBox b=surface.expolygon.contour.bounding_box(); fprintf(f,"FS int min=%ld %ld max=%ld %ld\n",(long)b.min.x(),(long)b.min.y(),(long)b.max.x(),(long)b.max.y()); fclose(f);} mb.unlock(); }'
    '';
  })
