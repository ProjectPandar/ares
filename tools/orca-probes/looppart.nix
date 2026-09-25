let pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/TriangleMeshSlicer.cpp \
        --replace-fail '                polygons = make_loops(lines[line_idx]);' '                polygons = make_loops(lines[line_idx]);
                if (getenv("ORCA_DUMP_LOOPPART") != nullptr) { FILE* f = fopen(getenv("ORCA_DUMP_LOOPPART"), "a"); if (f) { fprintf(f, "L%d ", int(line_idx)); for (const Polygon& pg : polygons) { fprintf(f,"["); for (const Point& q : pg.points) fprintf(f, "%ld,%ld;", (long)q.x(), (long)q.y()); fprintf(f,"]"); } fprintf(f,"\n"); fclose(f); } }'
    '';
  })
