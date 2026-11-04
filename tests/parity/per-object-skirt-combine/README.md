# Combined-brim, multi-instance per-object skirts

Source boundary: `docs/architecture/per-object-skirt-guard-retirement.md`.
Only inputs are committed; the parity harness produces native/API/CLI
outputs and comparison receipts beneath `ARES_PARITY_ARTIFACT_ROOT`.

Both inputs derive from `../per-object-skirt-brim/near-brim.3mf`, retaining
its mesh and stock profiles: Creality Ender-3 0.4 nozzle, 0.20mm Standard,
and Creality Generic PLA. The deliberate feature overrides are
`skirt_type=perobject`, two loops, height one, distance six, outer brim width
eight, zero brim gap, and `combine_brims=1`. No support, raft, tower, draft
shield, or minimum skirt length is enabled.

Relative to that parent, change only `combine_brims` from zero to one;
replace the build items with object IDs `[2, 2, 3]`, translations below;
and add object 2's instance ID 1 / identify ID 16 to the plate metadata.
Each build translation has Y=110 and Z=2. Object component transforms,
shared mesh, and other settings are unchanged.

| Case | X translations | Native skirt groups | Native brim blocks |
| --- | --- | --- | --- |
| `multi-instance-far` | 60, 100, 140 | 2 | 2 |
| `multi-instance-contact` | 80, 98, 116 | 1 | 1 |

Exact input hashes are pinned in `manifest.json`. Native double runs are
body-identical apart from the generator timestamp. The RED is the public
API/CLI unsupported-feature error; GREEN requires strict ordered full-output
parity and raw API/CLI equality, not merely successful admission.

With the established official Orca environment and artifact root configured:

```sh
ARES_REQUIRE_ORCA=1 cargo nextest run --no-fail-fast \
  -E 'test(per_object_skirt_combine)'
```
