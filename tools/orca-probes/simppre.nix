let
  pkgs = import <nixpkgs> {};
in
  pkgs.orca-slicer.overrideAttrs (old: rec {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/libslic3r/LayerRegion.cpp \
        --replace-fail 'void LayerRegion::simplify_path(ExtrusionPath* path)
{' 'void LayerRegion::simplify_path(ExtrusionPath* path)
{
    if (getenv("ORCA_DUMP_SIMPPRE") != nullptr) { static std::mutex msp; msp.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SIMPPRE"), "a"); if (f) { fprintf(f, "PRE role=%d n=%zu\n", (int)path->role(), path->polyline.points.size()); for (const auto& p : path->polyline.points) fprintf(f, "P %lld %lld\n", (long long)p.x(), (long long)p.y()); fclose(f);} msp.unlock(); }' \
        --replace-fail 'void LayerRegion::simplify_multi_path(ExtrusionMultiPath* multipath)
{' 'void LayerRegion::simplify_multi_path(ExtrusionMultiPath* multipath)
{
    if (getenv("ORCA_DUMP_SIMPPRE") != nullptr) { static std::mutex mmp; mmp.lock(); FILE* f = fopen(getenv("ORCA_DUMP_SIMPPRE"), "a"); if (f) { for (size_t pi = 0; pi < multipath->paths.size(); ++pi) { fprintf(f, "PRE role=%d n=%zu\n", (int)multipath->paths[pi].role(), multipath->paths[pi].polyline.points.size()); for (const auto& p : multipath->paths[pi].polyline.points) fprintf(f, "P %lld %lld\n", (long long)p.x(), (long long)p.y()); } fclose(f);} mmp.unlock(); }'
    '';
  })
