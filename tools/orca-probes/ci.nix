let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Fill/FillConcentricInternal.cpp \
        --replace-fail '        std::vector<const Arachne::ExtrusionLine*> all_extrusions;' '        if (getenv("ORCA_DUMP_CI") != nullptr) { static std::mutex cb; cb.lock(); FILE* f = fopen(getenv("ORCA_DUMP_CI"), "a"); if (f) { fprintf(f, "CI exp=%zu spacing=%ld loops=%ld\n", i, (long)min_spacing, (long)loops_count); for (auto &vw : loops) for (auto &elv : vw) { fprintf(f, "CIEXT closed=%d n=%zu w=[", (int)elv.is_closed, elv.junctions.size()); for (auto &j : elv.junctions) fprintf(f, "%.6f,", unscale_(j.w)); fprintf(f, "] p0=%ld,%ld\n", (long)elv.junctions.front().p.x(), (long)elv.junctions.front().p.y()); } fclose(f); } cb.unlock(); }
        std::vector<const Arachne::ExtrusionLine*> all_extrusions;'
    '';
  })
