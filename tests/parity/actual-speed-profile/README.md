# Actual speed profile live Native source-kernel authority

Only the hand-written input `profile.gcode` lives here. Native JSON, CSV, TSV
projections and run logs are written to unique **external** artifact
directories by `tools/chain-probe/actual-speed-profile-fixtures.mjs`, never
read from Git.

Approved boundary: `docs/architecture/web-actual-speed-profile.md`. Pinned
upstream: `8500fcdccaa10b5099ac20d252af3a7c560046f1`. This is a **Native
source-kernel diagnostic, not full Native Viewer/GL execution**: real
`GCodeProcessor` and `libvgcode::convert`, then the extracted
`GCodeViewer.cpp:1682-1752` block (`tools/chain-probe/actual-speed-profile/`).

```sh
export ORCA_PROCGCODE_SOURCE=${TMP_ROOT}/ares-orca-procgcode-source
export ORCA_PROCGCODE_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source
node tools/chain-probe/extract-actual-speed-profile.mjs --check
out=$(nix build --impure --cores 4 --max-jobs 1 --no-link --print-out-paths -f tools/chain-probe/actual-speed-profile.nix)
export ARES_ACTUAL_SPEED_PROFILE_BIN="$out/bin/orca-slicer"
ARES_REQUIRE_ORCA=1 cargo nextest run -p ares-vgcode --test actual_speed_profile
```
