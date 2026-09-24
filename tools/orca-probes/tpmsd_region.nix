let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillTpmsD.cpp \
        --replace-fail '    BoundingBox bb = expolygon.contour.bounding_box();' '    if (getenv("ORCA_DUMP_TPMSD") != nullptr) { static std::mutex mb; mb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_TPMSD"), "a"); if (f) { BoundingBox pbb = expolygon.contour.bounding_box(); fprintf(f, "TPMSD z=%.4f angle=%.5f dens=%.5f ml=%d min=%ld %ld max=%ld %ld npts=%zu\n", this->z, this->angle, params.density, params.multiline, (long)pbb.min.x(), (long)pbb.min.y(), (long)pbb.max.x(), (long)pbb.max.y(), expolygon.contour.points.size()); for (const Point &p : expolygon.contour.points) fprintf(f, "P %ld %ld\n", (long)p.x(), (long)p.y()); fclose(f); } mb.unlock(); }
    BoundingBox bb = expolygon.contour.bounding_box();'
    '';
  })
