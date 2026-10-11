# Merged Native oracle: the seed-only replay RNG plus every chain-probe
# dump flag (--process-gcode/--dump-moves, --dump-move-dynamics,
# --dump-marker-time, --dump-color-change, plate_N.print_statistics.json)
# plus the OptionsZCorrector m_move_id remap and the concentric/object-id
# initializations, in a single build. The source tree is assembled by
# scripts/ci/build-oracle.sh (git-applied patches + copied marker-time
# headers) because seed-only.patch needs --unidiff-zero application.
# Same ORCA_PROCGCODE_SOURCE / ORCA_PROCGCODE_NIXPKGS environment as
# procgcode.nix.
(import ./procgcode.nix).overrideAttrs (old: {
  pname = "orca-slicer-merged-oracle";
})
