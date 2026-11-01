let
  source = builtins.getEnv "ORCA_KSR_INITIALIZED_SOURCE";
  nixpkgs = builtins.getEnv "ORCA_KSR_INITIALIZED_NIXPKGS";
in
(import ../chain-probe/orca-beta.nix {
  inherit source nixpkgs;
  pname = "orca-slicer-ksr-initialized";
})
