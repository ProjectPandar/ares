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
   KSR, clippy and format.

Starting evidence: count-2 honeycomb fails at line 747 in the verified
artifacts from the preceding unit fix. Clipper-6 uses absolute arc tolerance
2 and rounded step counts; Clipper2 defaults to relative tolerance 0.002
and ceil steps. A standalone probe captures source-executed coordinates,
not a source-text pinning test.

## Verification

The red test records the original 183-versus-68 vertex failure. Four
focused tests pass after the fix.

The final live test passes all honeycomb counts 2/3/4 with exported
configuration assertions. The earlier run also passed, but count 4 needed
an exact-reference retry; stability is not established. The independent
bounded fill_multiline sweep recorded 3 generated, 3 strictly compared,
0 rejected and PASS. Neither result establishes full cross-product
coverage of patterns, printers and options.

Existing boolean union remains the explicitly deferred Clipper2 boundary.
Lightning and other full-goal residuals are not closed by this slice.
