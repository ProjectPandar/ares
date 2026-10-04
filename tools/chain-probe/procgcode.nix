# Standalone GCodeProcessor oracle (`--process-gcode <file>` and
# `--dump-moves <csv>`), built from a clean upstream 8500fcdcc worktree with
# procgcode.patch. Usage (same pinned nixpkgs as scripts/orca-replay/build.nix):
#
#   src=$(mktemp -d /tmp/ares-orca-procgcode-source.XXXXXX)
#   git -C OrcaSlicer worktree add --detach "$src" 8500fcdcc
#   ORCA_PROCGCODE_SOURCE="$src" \
#   ORCA_PROCGCODE_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source \
#   nix build --impure --no-link --print-out-paths --file tools/chain-probe/procgcode.nix
let
  source = builtins.getEnv "ORCA_PROCGCODE_SOURCE";
  nixpkgs = builtins.getEnv "ORCA_PROCGCODE_NIXPKGS";
  pkgs = import nixpkgs {};
in
assert source != "";
assert nixpkgs != "";
pkgs.orca-slicer.overrideAttrs (old: {
  pname = "orca-slicer-procgcode";
  src = builtins.path {
    path = builtins.toPath source;
    name = "orca-slicer-${old.version}-procgcode-source";
    filter = path: type: builtins.baseNameOf path != ".git";
  };
  patches = (old.patches or []) ++ [ ./procgcode.patch ];
})
