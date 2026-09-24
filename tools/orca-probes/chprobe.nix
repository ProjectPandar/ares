let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillCrossHatch.cpp \
        --replace-fail '    Polylines polylines = generate_infill_layers(scale_(this->z), repeat_ratio, line_spacing, bb.size()(0), bb.size()(1));' '    Polylines polylines = generate_infill_layers(scale_(this->z), repeat_ratio, line_spacing, bb.size()(0), bb.size()(1));
        if (getenv("ORCA_DUMP_CH")!=nullptr){ static std::mutex mc; mc.lock(); FILE*f=fopen(getenv("ORCA_DUMP_CH"),"a"); if(f){ fprintf(f,"CH z=%f ls=%ld rr=%f bbmin=%ld,%ld w=%ld h=%ld npol=%zu\n",(double)this->z,(long)line_spacing,repeat_ratio,(long)bb.min.x(),(long)bb.min.y(),(long)bb.size()(0),(long)bb.size()(1),polylines.size()); for(size_t i=0;i<polylines.size()&&i<3;i++){const Polyline&p=polylines[i]; fprintf(f,"  pl%zu:",i); for(const Point&pt:p.points)fprintf(f," %ld,%ld",(long)pt.x(),(long)pt.y()); fprintf(f,"\n");} fclose(f);} mc.unlock(); }'
    '';
  })
