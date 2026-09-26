# Multiline round-offset fidelity

## Boundary

Port the constant positive-delta, round-join/round-end open-path subset of
vendored Clipper2 `clipper.offset.cpp` (`BuildNormals`, `OffsetPoint`,
`DoRound`, `OffsetOpenPath`, `DoGroupOffset`) reached by
`FillBase.cpp::multiline_fill`. Rust destination:
`fill/multiline_offset/round.rs`, called only by `multiline_offset::apply`.
The current Clipper-6 open-offset call is a temporary compatibility shell,
not the owning upstream implementation for this call site.

Included: default relative arc tolerance 0.002 times delta, ceil-based
arc steps, offset-vector rotation, round caps, concave joins, duplicate
point removal and one coordinate scaling at the caller. Existing boolean
union remains a compatibility boundary. Full Clipper2 boolean engine,
other join/end modes, callback/negative offsets and unrelated Clipper-6
users are deferred. No acceptance tolerance or alternate fallback.

## Plan and evidence

1. Compile a standalone probe against vendored Clipper2; regress actual
   multiline output for an L-shaped polyline, delta 200000: 67 polygon
   vertices including (1025235,-198402), plus the closing point.
2. Replace the incompatible Clipper-6 round generation at this seam;
   verify duplicates, reflected coordinates, odd/even bundles and caps.
3. Run the honeycomb multiline live test (all 2/3/4 counts, exported
   configuration assertions and complete paired output). Verify workspace,
   KSR, clippy and format; independent review, fix loop and code+docs push.

Starting evidence: count-2 honeycomb fails at line 747 in the verified
artifacts from the preceding unit fix. Clipper-6 uses absolute arc tolerance
2 and rounded step counts; Clipper2 defaults to relative tolerance 0.002
and ceil steps. `/tmp/multiline-round-probe.cpp` and its output capture
source-executed coordinates, not a source-text pinning test.

## Verification

`/tmp/multiline-round-red.log` records the original 183-versus-68 vertex
failure. Four focused tests pass in `/tmp/multiline-round-green.log`.
After removing two newly introduced clippy warnings, final workspace
`/tmp/multiline-round-final-workspace.log` records 7326 passed / 10 skipped;
final KSR records 84 passed / 2 skipped. Final clippy completes without
warnings naming the changed seam; fmt and diff checks pass.

The final live test `/tmp/multiline-round-final-oracle.log` passes all
honeycomb counts 2/3/4 with exported configuration assertions. Complete
paired bytes are under `/tmp/parity-multiline-round-final`. The earlier
run also passed, but count 4 needed an exact-reference retry; stability
is not established. The independent bounded fill_multiline sweep recorded
3 generated, 3 strictly compared, 0 rejected and PASS under
`/tmp/parity-multiline-round-domain`. Neither result establishes full
cross-product coverage of patterns, printers and options.

Independent six-axis reviewer `65b1d130` found no scoped issues; final
oracle/KSR completion was verified by the parent after that review.
Existing boolean union remains the explicitly deferred Clipper2 boundary.
Lightning and other full-goal residuals are not closed by this slice.
