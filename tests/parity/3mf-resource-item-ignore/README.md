# Resource build/item ignore — frozen public slicing witness

SPEC `cbd9b2c86a2cf3a3154d947d722092a4d8fa90cd`.
Orca source8500fcdccaa10b5099ac20d252af3a7c560046f1:
`Format/bbs_3mf.cpp::ObjectImporter` active workers1780-1817 and resource
start/end dispatch5612-5687 do not dispatch build/item; main item3893-3909 is
separate. This is ignored resource payload, not main schema relaxation.

## Input-only frozen pair

- control.3mf SHA256
  `adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2`
  exact `wipe-before-override/control-before50.3mf`.
- positive.3mf SHA256
  `41dca2941a280ed674b063bed0cbb5afd9d37336c4757f8327afc331e39d1464`.

ONLY uncompressed ZIP entry `3D/Objects/overhang.stl_1.model` changes:
replace its single `<build/>` with `<build><item/></build>`. Main build,
mesh/roles/IDs/transforms/settings/M73 and every other entry are byte-identical.
Manifest pins both archives and before/after SHA of every entry. The empty item
is well-formed XML, deliberately missing a main-item attribute; not claimed
schema-valid as a main item. No oracle G-code/output/cache is committed.

## Runtime contract

`resource_item_ignored_public_slicing`, enabled with the existing
`ARES_PARITY_PROFILE_ENTRY_REFERENCE`/`OrcaRunner::from_env` contract, always
creates three fresh Native --slice references (positive/repeat/control), then
validated public API + real CLI on the same input bytes. No Native-only switch,
external reference injection or cached/export/load-only substitute.
Native `--slice 0 --outputdir`, no arrange flag, loaded JSONs or extra overrides.
Require complete exit0/footer and generator-only Native repeat/control neutrality.
API uses default overrides, plateNone, contextseed0 and matching deterministic
metadata. CLI uses bare `ares slice -o <fresh> <same frozen input>`.

All three public records persist before assertions. Control must first admit,
pass raw API==CLI and the entire unchanged strict generator-only Native/API/CLI
comparator (M73/config/motion/arcs/statistics/footer retained). Positive/repeat
assert successful admission and the same complete comparisons: currently real
RED is `InvalidInput("invalid project model XML: missing field \`@objectid\`")`
with CLIexit1 samecause versus Native complete G-code. No Ares output means
raw equality/comparison null/NOT EVALUATED, never missing-output equality PASS.
Assertions require eventual correct behavior, not permanently expected errors.

Run official and seeded with fresh cache/artifact/TMPDIR and canonical
environments. Use `cargo nextest run -p ares-cli --test orca_parity -E
 'test(resource_item_ignored_public_slicing)' --test-threads 1`.
Any control/full-output difference => STOP/report, no retries/input substitutions.
No M73 work.
