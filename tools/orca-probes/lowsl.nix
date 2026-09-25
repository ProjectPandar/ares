let pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/PerimeterGenerator.cpp \
        --replace-fail '        m_lower_slices_polygons = offset(*this->lower_slices, float(scale_(+nozzle_diameter / 2)));' '        m_lower_slices_polygons = offset(*this->lower_slices, float(scale_(+nozzle_diameter / 2)));
        if (getenv("ORCA_DUMP_LOWSL") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_LOWSL"), "a"); if (f) { fprintf(f, "L%d ", int(this->layer_id)); for (const ExPolygon& e : *this->lower_slices) { fprintf(f,"["); for (const Point& q : e.contour.points) fprintf(f, "%ld,%ld;", (long)q.x(), (long)q.y()); fprintf(f,"]"); } fprintf(f,"\n"); fclose(f); } }'
    '';
  })
