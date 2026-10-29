# Per-group minimum skirt filament inputs

Approved SPEC: `docs/architecture/per-object-skirt-minimum-length-spec.md`
(`3cfc39566`). Pure-input 3MFs only; no checked-in Native outputs.

Eight exact inputs/hashes are in `manifest.json`. Each pair preserves the
original disjoint/overlap/transitive/copies fixture's mesh, identities,
transforms and other settings, changing only min_skirt_length 0 -> 20.
Native discovery `/tmp/ares-skirt-min-length-allgroups-37UMhP/` accepted all
16 serial official/seeded exports with sampled CPU18 affinity. Group counts
remain 2/1/1/1. The two-loop controls each have 42 positive XY skirt segments;
20mm inputs have extra loops and actual positive extrusion independently in
every group (disjoint 8 loops/group, overlap 6, transitive 5, copies 4).
Export seam clipping means actual emitted E is not the general definition of
minimum length. The observed >20mm emitted E assertion is specific to these
fixed inputs, not an alternative implementation of the source budget.

Public test selection: `test(orca_parity_per_object_skirt_min_length_)`.
Run with the established required Native environment, fresh caches/artifacts,
isolated target policy and CPU18. Tests retain Native admission, full output,
actual motion witness, exact input/producer hashes before public Ares API/CLI
assertions. They require success and existing strict ordered Native/API/CLI
comparisons plus raw API/CLI equality; rejection is never asserted as success.
Baseline positive-input failure is retained as RED, pending review before
production changes. No fixtures/assertions are weakened during GREEN.
