let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/Fill.cpp \
        --replace-fail '            double ironing_areas_offset = ironing_params.inset == 0 ? float(scale_(0.5 * nozzle_dmr)) : scale_(ironing_params.inset);
			ironing_areas = intersection_ex(polys, offset(this->lslices, - ironing_areas_offset));' '            double ironing_areas_offset = ironing_params.inset == 0 ? float(scale_(0.5 * nozzle_dmr)) : scale_(ironing_params.inset);
			if (getenv("ORCA_DUMP_IRON")!=nullptr){ static std::mutex mi; mi.lock(); FILE*f=fopen(getenv("ORCA_DUMP_IRON"),"a"); if(f){ BoundingBox bp=get_extents(polys), bl=get_extents(this->lslices), bo=get_extents(offset(this->lslices, - ironing_areas_offset)); fprintf(f,"IRON lid=%d polys=[%ld %ld %ld %ld] lslices=[%ld %ld %ld %ld] offset=[%ld %ld %ld %ld] inset=%f\n",(int)this->id(),(long)bp.min.x(),(long)bp.min.y(),(long)bp.max.x(),(long)bp.max.y(),(long)bl.min.x(),(long)bl.min.y(),(long)bl.max.x(),(long)bl.max.y(),(long)bo.min.x(),(long)bo.min.y(),(long)bo.max.x(),(long)bo.max.y(),ironing_areas_offset); fclose(f);} mi.unlock(); }
			ironing_areas = intersection_ex(polys, offset(this->lslices, - ironing_areas_offset));
			if (getenv("ORCA_DUMP_IRON")!=nullptr){ static std::mutex mi2; mi2.lock(); FILE*f=fopen(getenv("ORCA_DUMP_IRON"),"a"); if(f){ BoundingBox bi=get_extents(ironing_areas); fprintf(f,"IRON lid=%d areas=[%ld %ld %ld %ld]\n",(int)this->id(),(long)bi.min.x(),(long)bi.min.y(),(long)bi.max.x(),(long)bi.max.y()); fclose(f);} mi2.unlock(); }'
    '';
  })
