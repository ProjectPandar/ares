Nix `pkgs.orca-slicer.overrideAttrs` probes that patch upstream OrcaSlicer to
dump internal state for parity bisection.

- orcablocks.nix  ORCA_DUMP_BLOCKS -> per-TimeBlock `id dist cruise accel entry exit safe time`
- arcseg.nix      ORCA_DUMP_ARC    -> per-arc-segment `SEG x y z e` target (process_G1 args)
- simppath.nix    ORCA_DUMP_SIMP   -> erBottomSurface path PRE/POST simplify points
- hilbert_preclip.nix ORCA_DUMP_PRECLIP -> plane-path IN/OUT clip polylines

Build: `nix-build tools/orca-probes/NAME.nix --no-out-link`, run the result's
`bin/orca-slicer --slice 0 --outputdir DIR input.3mf` with the env var set.
- lslices.nix    ORCA_DUMP_LSLICES -> merged Layer::lslices contour vertices (post make_slices)

## Throwaway Rust probes (never commit)

For probing Ares internals on another branch (e.g. an O3 fix not yet on
main), use a detached temp worktree plus a local-only integration test:

1. `git worktree add --detach <probe-dir> <branch-or-commit>`
2. Write `crates/ares-core/tests/zz_tmp_probe.rs`: build a
   `ProfileSliceRequest` from a saved runner directory's
   `catalog-array.json`/`selection.json`/model, call `profile_settings`,
   `profile_placed_project`, or `slice_with_profiles`, and `println!` the
   typed fields or emitted lines of interest.
3. `cd <probe-dir> && CARGO_TARGET_DIR=<main worktree>/target
   cargo nextest run -p ares-core --test zz_tmp_probe --no-capture`
   (shared target dir avoids a second full build).
4. Delete the test file and `git worktree remove --force` when done.

The `zz_tmp_probe.rs` file is a local instrument only — never commit it;
record durable findings in the spec instead.
