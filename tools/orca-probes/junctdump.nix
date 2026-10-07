let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/Arachne/WallToolPaths.cpp \
        --replace-fail 'void WallToolPaths::simplifyToolPaths(std::vector<VariableWidthLines>& toolpaths, const WallToolPathsParams& params)
{' 'void WallToolPaths::simplifyToolPaths(std::vector<VariableWidthLines>& toolpaths, const WallToolPathsParams& params)
{
    if (getenv("ORCA_DUMP_JUNCT") != nullptr) { static std::mutex jm; jm.lock(); FILE* f = fopen(getenv("ORCA_DUMP_JUNCT"), "a"); if (f) { for (const VariableWidthLines& lines : toolpaths) for (const ExtrusionLine& line : lines) { fprintf(f, "PRE inset=%d closed=%d nj=%zu", line.inset_idx, (int)line.is_closed, line.junctions.size()); for (const ExtrusionJunction& j : line.junctions) fprintf(f, " %lld,%lld,w%lld", (long long)j.p.x(), (long long)j.p.y(), (long long)j.w); fprintf(f, "\n"); } fclose(f); } jm.unlock(); }' \
        --replace-fail '            line.simplify(smallest_line_segment_squared, allowed_error_distance_squared, maximum_extrusion_area_deviation);' '            line.simplify(smallest_line_segment_squared, allowed_error_distance_squared, maximum_extrusion_area_deviation);
            if (getenv("ORCA_DUMP_JUNCT") != nullptr) { static std::mutex jm2; jm2.lock(); FILE* f = fopen(getenv("ORCA_DUMP_JUNCT"), "a"); if (f) { fprintf(f, "POST nj=%zu", line.junctions.size()); for (const ExtrusionJunction& j : line.junctions) fprintf(f, " %lld,%lld,w%lld", (long long)j.p.x(), (long long)j.p.y(), (long long)j.w); fprintf(f, "\n"); fclose(f); } jm2.unlock(); }'
    '';
  })
