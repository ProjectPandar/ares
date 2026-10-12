# Plate custom G-codes oracle (docs/architecture/project-plate-custom-gcodes.md): the seeded
# replay producer source plus plate-custom-gcodes.patch, which with
# ARES_PLATE_CUSTOM_GCODES_DUMP=<path> writes Model::plates_custom_gcodes right after
# Model::read_from_file (OrcaSlicer.cpp:1640) and exits.
#
#   ORCA_PLATE_CUSTOM_GCODES_SOURCE=/nix/store/g3wj93lyhjsvc32fmgvw0cdbd6191q55-orca-slicer-2.4.2-seed-replay-source \
#   ORCA_PLATE_CUSTOM_GCODES_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source \
#   nix build --impure --cores 4 --max-jobs 1 --no-link --print-out-paths --file tools/chain-probe/plate-custom-gcodes.nix
let
  source = builtins.getEnv "ORCA_PLATE_CUSTOM_GCODES_SOURCE";
  nixpkgs = builtins.getEnv "ORCA_PLATE_CUSTOM_GCODES_NIXPKGS";
  pkgs = import nixpkgs {};
in
assert source != "";
assert nixpkgs != "";
pkgs.orca-slicer.overrideAttrs (old: {
  pname = "orca-slicer-plate-custom-gcodes";
  src = builtins.path {
    path = builtins.toPath source;
    name = "orca-slicer-${old.version}-seed-replay-source";
    filter = path: type: builtins.baseNameOf path != ".git";
  };
  patches = (old.patches or []) ++ [ ./plate-custom-gcodes.patch ];
})
