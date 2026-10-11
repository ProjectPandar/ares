# Historical initialized KSR producer; instantiate without building:
# ORCA_KSR_INITIALIZED_SOURCE=/path/to/restored-source nix-instantiate tools/orca-probes/ksr-initialized.nix
# Upstream 8500fcdcc + ksr-initialized-source.patch (not current seed-only.patch).
let
  source = builtins.getEnv "ORCA_KSR_INITIALIZED_SOURCE";
  nixpkgs = builtins.getEnv "ORCA_KSR_INITIALIZED_NIXPKGS";
  pkgs = import nixpkgs {};
in
assert source != "";
assert nixpkgs != "";
pkgs.orca-slicer.overrideAttrs (old: {
  pname = "orca-slicer-seed-replay";
  src = builtins.path {
    path = builtins.toPath source;
    name = "orca-slicer-${old.version}-seed-replay-source";
    filter = path: type: builtins.baseNameOf path != ".git";
  };
})
