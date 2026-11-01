let
  source = builtins.getEnv "ORCA_PLATE_CUSTOM_GCODES_SOURCE";
  nixpkgs = builtins.getEnv "ORCA_PLATE_CUSTOM_GCODES_NIXPKGS";
in
(import ./orca-beta.nix {
  inherit source nixpkgs;
  pname = "orca-slicer-plate-custom-gcodes";
}).overrideAttrs (old: {
  patches = old.patches ++ [ ./plate-custom-gcodes.patch ];
})
