{ pkgs }:
let
  patched = pkgs.orca-slicer.overrideAttrs (old: {
    nativeBuildInputs = (old.nativeBuildInputs or []) ++ [ pkgs.python3 ];
    postPatch = (old.postPatch or "") + ''
python3 << 'PYEOF'
import sys
p = "src/libslic3r/GCode/SeamPlacer.cpp"
s = open(p).read()
anchor = """        perimeter.finalized = true;
      }

#ifdef DEBUG_FILES"""
probe = """        perimeter.finalized = true;
      }
      if (getenv("ORCA_DUMP_STR")) {
        static FILE *strf = fopen(getenv("ORCA_DUMP_STR"), "a");
        fprintf(strf, "STR start=%f,%f len=%zu\\n",
                layers[seam_string[0].first].points[seam_string[0].second].position.x(),
                layers[seam_string[0].first].points[seam_string[0].second].position.y(),
                seam_string.size());
        for (size_t i = 0; i < seam_string.size(); ++i) {
          const Perimeter &pp = layers[seam_string[i].first].points[seam_string[i].second].perimeter;
          const SeamCandidate &cc = layers[seam_string[i].first].points[seam_string[i].second];
          fprintf(strf, "  (%zu,%zu) %f,%f,%f si=%zu final=%f,%f,%f\\n",
                  seam_string[i].first, seam_string[i].second,
                  cc.position.x(), cc.position.y(), cc.position.z(),
                  pp.seam_index,
                  pp.final_seam_position.x(), pp.final_seam_position.y(), pp.final_seam_position.z());
        }
        fflush(strf);
      }

#ifdef DEBUG_FILES"""
assert anchor in s, "anchor miss"
s = s.replace(anchor, probe, 1)
open(p, "w").write(s)
PYEOF
    '';
  });
in
patched
