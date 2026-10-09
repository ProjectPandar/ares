# Summary oracle (docs/architecture/web-summary.md): the seeded replay producer
# (1k0p5nlb…-orca-slicer-seed-replay-2.4.2, same source store path) plus
# print-statistics.patch, which with --outputdir <dir> also writes
# <dir>/plate_<n>.print_statistics.json after export: the GUI PrintStatistics totals
# (GCode.cpp:1904-1940) and their GCodeProcessorResult sources.
#
#   ORCA_PRINT_STATISTICS_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source \
#   nix build --impure --cores 8 --no-link --print-out-paths --file tools/chain-probe/print-statistics.nix
let
  nixpkgs = builtins.getEnv "ORCA_PRINT_STATISTICS_NIXPKGS";
  pkgs = import nixpkgs {};
in
assert nixpkgs != "";
pkgs.orca-slicer.overrideAttrs (old: {
  pname = "orca-slicer-print-statistics";
  src = builtins.storePath /nix/store/g3wj93lyhjsvc32fmgvw0cdbd6191q55-orca-slicer-2.4.2-seed-replay-source;
  patches = (old.patches or []) ++ [ ./print-statistics.patch ];
})
