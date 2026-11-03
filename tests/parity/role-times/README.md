# Role times live Native source-kernel authority

Only the hand-written input `roles.gcode` lives here. Native JSON, CSV, TSV
projections and run logs are written to unique **external** artifact
directories by `tools/chain-probe/role-times-fixtures.mjs`, never read from Git.

Approved boundary: `docs/architecture/web-role-times.md`. Pinned upstream:
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. This is a **Native source-kernel
diagnostic, not full Native Viewer/GL execution**: real `GCodeProcessor`,
`libvgcode::convert` and `libvgcode::ExtrusionRoles`, then the extracted
`ViewerImpl.cpp:1002-1018` accumulators and `GCodeViewer.cpp:3151, 3410-3418,
3638-3668` Feature Type cells (`tools/chain-probe/role-times/`).

```sh
export ORCA_PROCGCODE_SOURCE=${TMP_ROOT}/ares-orca-procgcode-source
export ORCA_PROCGCODE_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source
node tools/chain-probe/extract-role-times.mjs --check
out=$(nix build --impure --cores 4 --max-jobs 1 --no-link --print-out-paths -f tools/chain-probe/role-times.nix)
export ARES_ROLE_TIMES_BIN="$out/bin/orca-slicer"
ARES_REQUIRE_ORCA=1 cargo nextest run -p ares-vgcode --test extrusion_roles
```
