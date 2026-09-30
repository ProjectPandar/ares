let
  source = builtins.getEnv "ORCA_REPLAY_SOURCE";
  nixpkgs = builtins.getEnv "ORCA_REPLAY_NIXPKGS";
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
