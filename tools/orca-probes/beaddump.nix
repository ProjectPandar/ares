let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Arachne/BeadingStrategy/DistributedBeadingStrategy.cpp \
        --replace-fail 'ret.total_thickness = thickness;' 'if (getenv("ORCA_DUMP_BEADING") != nullptr) { fprintf(stderr, "BEAD t=%lld n=%lld\n", (long long)thickness, (long long)bead_count); } ret.total_thickness = thickness;'
    '';
  })
