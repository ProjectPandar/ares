let
  source = builtins.getEnv "ORCA_PRINT_STATISTICS_SOURCE";
  nixpkgs = builtins.getEnv "ORCA_PRINT_STATISTICS_NIXPKGS";
in
(import ./orca-beta.nix {
  inherit source nixpkgs;
  pname = "orca-slicer-print-statistics";
}).overrideAttrs (old: {
  patches = old.patches ++ [ ./print-statistics.patch ];
})
