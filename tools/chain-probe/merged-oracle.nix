let
  source = builtins.getEnv "ORCA_PROCGCODE_SOURCE";
  nixpkgs = builtins.getEnv "ORCA_PROCGCODE_NIXPKGS";
in
(import ./orca-beta.nix {
  inherit source nixpkgs;
  pname = "orca-slicer-merged-oracle";
})
