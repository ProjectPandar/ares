# Proposed retirement of the obsolete Arachne alternate rejection test

Status: coordinator approved retiring this one obsolete test after RED8989e26eb.

The existing test is
`crates/ares-core/src/project_slice/tests/perimeters/arachne/config.rs`,
`task22w9_rejects_deferred_scopes_typed` (lines 87–101 on c22f5a81d).
Its only executable assertion sets alternate extra wall true, density 15,
and expects layer 1 configuration to return
`UnsupportedProjectFeature("alternate_extra_wall")`.

That assertion describes the scaffold being replaced, not an upstream rejection.
`PerimeterGenerator.cpp::process_arachne:2131–2134` admits this combination and
increments the loop count. Lines 2136–2144 and 2163–2175 apply the existing
overrides and generator counts after it. The approved SPEC requires deleting
the production guard; leaving this rejection assertion would require retaining
the forbidden guard or failing workspace by design.

Approved concrete change:

- Delete this obsolete test function and its now-unused `Percent` import.
- Preserve every other existing assertion, fixture and comparator.
- Keep the new public Native/API/CLI E2E on/off cases as the replacement
  contract: odd/even counts, integer density 0/0.5/1, first/top overrides,
  top regeneration, zero configured walls, independent islands and spiral.
  New tests and Native RED are committed before production changes.

The first Native preflight confirmed odd-layer wall crossings 4→6 for the
ordinary positive-density prism and independent-island crossings 8→12, while
Ares returned exactly the above UnsupportedProjectFeature. Density 0.5 was
Native-admitted and inert but rejected by Ares. Raw preflight evidence remains
outside Git at `/tmp/worker3-arachne-alternate-red-sui9f3ez`.

This is a request to replace an obsolete negative contract with upstream
positive public coverage; it does not authorize weakening a parity assertion.
Coordinator explicitly selected “批准退役这一过时拒绝测试” on 2026-10-13.
