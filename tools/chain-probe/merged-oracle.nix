# Merged Native oracle: the seed-only replay RNG plus every chain-probe
# dump flag (--process-gcode/--dump-moves, --dump-move-dynamics,
# --dump-marker-time, --dump-color-change, plate_N.print_statistics.json)
# plus the OptionsZCorrector m_move_id remap and the concentric/object-id
# initializations, in a single build. The source tree is assembled by
# scripts/ci/build-oracle.sh (git-applied patches + copied marker-time
# headers) because seed-only.patch and object-id-initialization.patch need
# --unidiff-zero application. Same ORCA_PROCGCODE_SOURCE /
# ORCA_PROCGCODE_NIXPKGS environment as procgcode.nix; no extra patches
# are added here — the derivation must not re-apply what the source tree
# already carries.
let
  source = builtins.getEnv "ORCA_PROCGCODE_SOURCE";
  nixpkgs = builtins.getEnv "ORCA_PROCGCODE_NIXPKGS";
  pkgs = import nixpkgs {};
in
assert source != "";
assert nixpkgs != "";
pkgs.orca-slicer.overrideAttrs (old: {
  pname = "orca-slicer-merged-oracle";
  src = builtins.path {
    path = builtins.toPath source;
    name = "orca-slicer-${old.version}-merged-oracle-source";
    filter = path: type: builtins.baseNameOf path != ".git";
  };
})
