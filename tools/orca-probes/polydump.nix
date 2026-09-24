let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Arachne/SkeletalTrapezoidation.cpp \
        --replace-fail 'void SkeletalTrapezoidation::constructFromPolygons(const Polygons& polys)
{' 'void SkeletalTrapezoidation::constructFromPolygons(const Polygons& polys)
{
    if (getenv("ORCA_DUMP_STPOLY") != nullptr) { size_t npts = 0; for (const auto& p : polys) npts += p.size(); fprintf(stderr, "STPOLY polys=%zu pts=%zu\n", polys.size(), npts); }'
    '';
  })
