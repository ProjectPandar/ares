# Initialized-native diagnostic oracle for MoveVertex pressure_advance /
# acceleration / jerk (docs/architecture/web-motion-attributes.md). Same
# ORCA_PROCGCODE_SOURCE / ORCA_PROCGCODE_NIXPKGS environment as procgcode.nix.
# On top of procgcode-stable-options it only adds `--dump-move-dynamics <csv>`,
# initializes the otherwise-uninitialized m_pressure_advance to 0 in
# GCodeProcessor::reset() (upstream UB; ruling: not ported, Ares records 0),
# and gates the travel acceleration limit by flavor in the process_file
# DynamicPrintConfig path exactly as the slicing PrintConfig path does
# (GCodeProcessor.cpp:2116-2119), so the oracle models a sliced preview.
(import ./procgcode-stable-options.nix).overrideAttrs (old: {
  pname = "orca-slicer-procgcode-dynamics";
  patches = old.patches ++ [ ./procgcode-dynamics.patch ];
})
