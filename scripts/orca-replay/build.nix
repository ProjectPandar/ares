let
  source = builtins.getEnv "ORCA_REPLAY_SOURCE";
  nixpkgs = builtins.getEnv "ORCA_REPLAY_NIXPKGS";
in
(import ../../tools/chain-probe/orca-beta.nix {
  inherit source nixpkgs;
  pname = "orca-slicer-seed-replay";
})
