let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/GCode/ExtrusionProcessor.hpp \
        --replace-fail '        prev_curled_extrusions[object] = next_curled_extrusions[object];' '        if (getenv("ORCA_DUMP_CURL") != nullptr) { static std::mutex cb; cb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_CURL"), "a"); if (f) { fprintf(f, "CURL layer=%zu n=%zu\n", layer->id(), layer->curled_lines.size()); for (const CurledLine &cl : layer->curled_lines) fprintf(f, " CL %.4f,%.4f-%.4f,%.4f h=%.6f\n", unscale_(cl.a.x()), unscale_(cl.a.y()), unscale_(cl.b.x()), unscale_(cl.b.y()), cl.curled_height); fclose(f); } cb.unlock(); }
        prev_curled_extrusions[object] = next_curled_extrusions[object];'
    '';
  })
