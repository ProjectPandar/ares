let
  pkgs = import <nixpkgs> {};
  dump2 = builtins.readFile ./dump2.txt;
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/PerimeterGenerator.cpp \
        --replace-fail \
        '                ex.medial_axis(min, max, &polylines);' \
        '                ex.medial_axis(min, max, &polylines);
'"$(cat ${./dump2.txt})"
    '';
  })
