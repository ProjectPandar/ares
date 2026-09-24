Nix `pkgs.orca-slicer.overrideAttrs` probes that patch upstream OrcaSlicer to
dump internal state for parity bisection.

- orcablocks.nix  ORCA_DUMP_BLOCKS -> per-TimeBlock `id dist cruise accel entry exit safe time`
- arcseg.nix      ORCA_DUMP_ARC    -> per-arc-segment `SEG x y z e` target (process_G1 args)
- simppath.nix    ORCA_DUMP_SIMP   -> erBottomSurface path PRE/POST simplify points
- hilbert_preclip.nix ORCA_DUMP_PRECLIP -> plane-path IN/OUT clip polylines

Build: `nix-build tools/orca-probes/NAME.nix --no-out-link`, run the result's
`bin/orca-slicer --slice 0 --outputdir DIR input.3mf` with the env var set.
