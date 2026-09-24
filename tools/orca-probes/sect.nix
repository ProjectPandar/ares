let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/Fill.cpp \
        --replace-fail '        polygon_sections = filter_vibrating_extrusions(polygon_sections);' '        polygon_sections = filter_vibrating_extrusions(polygon_sections);
        if (getenv("ORCA_DUMP_SECT") != nullptr) { static std::mutex sb; sb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SECT"), "a"); if (f) { for (size_t i = 0; i < polygon_sections.size(); i++) { fprintf(f, "SEC %zu n=%zu", i, polygon_sections[i].size()); for (const Line &l : polygon_sections[i]) fprintf(f, " %ld:%ld-%ld:%ld", (long)l.a.x(), (long)l.a.y(), (long)l.b.x(), (long)l.b.y()); fprintf(f, "\n"); } fclose(f); } sb.unlock(); }'
    '';
  })
