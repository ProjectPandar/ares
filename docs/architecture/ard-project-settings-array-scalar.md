# Project-settings array-valued scalar — Rust rewrite slice

Source RED parent:
`1f784dbae58dd1f27c51c688e68176a85623dbe9` (main=aecea85be lineage).
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Status: accepted. Chained RED→GREEN after the read-only pinpoint
(`support-pinpoint-array-scalar-aecea85be`).

## Owning source and non-negotiable boundary

Port the homogeneous-array branch of `ConfigBase::load_from_json`
(`Config.cpp:947-1000`) for scalar options inside
`ares-core/src/options/typed_legacy/`:

1. A JSON array of same-type string elements for a scalar rule joins via the
   existing vector machinery with the upstream scalar separator `','`
   (`Config.cpp:832-867 parse_str_arr`: no separator before the first
   element; nested arrays join with `'#'`).
2. The joined string then flows into each type's own deserialize semantics —
   there is deliberately **no generic "first token" rule**. Per
   `Config.hpp` per-type `deserialize`:
   - `ConfigOptionFloat::deserialize` (`Config.hpp:784-790`) is
     `std::istringstream >> value`: the first numeric token wins, any
     remainder after a non-numeric character (the joining `,`) is silently
     unconsumed, extraction succeeds. `coInt` follows the same
     istringstream pattern (`Config.hpp:859-866`).
   - `coString` keeps the whole joined string verbatim, commas included.
   - `coBool`/`coEnum` failures hand off to the existing substitution
     handling mirroring `Config.cpp:655-669` (default substituted, success,
     logged), reusing what Ares already implements for scalar strings.
3. Coverage is limited to the scalar kinds that this RED and the shared
   `WireContract::scalar` contract necessarily exercise; no speculative
   type mapping beyond them.

Witnesses: `tests/parity/project-settings/mk4-iface-speed-arr42.3mf`
(`support_interface_speed: ["42"]`) and
`mk4-iface-speed-arr42-77.3mf` (`["42","77"]`). Native (official 2.4.2 and
seeded) slices both, emitting `; support_interface_speed = 42`; the two
G-codes are byte-identical to the scalar `"42"` control and differ from the
unset(80) control — Native applies the array value with first-element
semantics. Ares today rejects both inputs
(`InvalidInput("invalid legacy Orca project array option support_interface_speed")`).

## GREEN implementation (chained on RED `1317de56e`)

The array join and the per-type split live at the boundaries the upstream
function itself separates:

1. `WireContract::scalar` now carries `JsonArrayAllowance::FlattenScalar`
   (`actions.rs`); `transform_json_array` routes it through the existing
   `flatten_json_array` machinery with the `VectorType::Ints` separator
   profile (`','`, nested `'#'`, no c-style quoting — quoting belongs to the
   `coStrings` vector type alone, `Config.cpp:974-987`).
2. `lexical::assign` with `ValueOrigin::Array` and a scalar target hands
   the joined string to `json_deserialize::scalar_token` — the same
   `scalar::normalize` port the preset presence view uses — so the
   per-type semantics have one source: numeric `istringstream` prefix
   (`coFloat`/`coInt`/`coPercent`/`coFloatOrPercent`), `coBool` first
   trimmed `','`-segment, everything else verbatim (strings keep the whole
   join; enums fall through to the existing `Config.cpp:657-676` default
   substitution on parse failure).
3. The embedded raw pipeline (`project/load.rs`) calls the new
   `ProjectSettingsRaw::normalize_scalar_arrays` — the existing
   `normalize_numeric_scalar_arrays` (already the preset-side port of this
   same branch) — after `handle_legacy`, mirroring the upstream order:
   the branch's first `handle_legacy(key, "")` rename, then the join, then
   `set_deserialize`'s second `handle_legacy(key, joined)` value rewrite.
   The `Slic3r::validate` flat map and the export view therefore carry the
   bound scalar, not the array.
4. Contract-snapshot tests (`typed_legacy/inventory.rs`, `direct_feature.rs`,
   `convert/arrays.rs`) updated to the new invariant: no rule keeps a
   first-pass array rejection, and `["top"]` on `top_one_wall_type` binds
   like the bare string.

Verification: official and seeded parity modes both run the
`project_settings` group 6/6 green (A/D witnesses PASS);
`cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`
clean; `options::tests::typed_legacy`, `options::json_deserialize`,
`options::tests::legacy`, `project::raw_settings` lib tests 129/129 green.
The two pre-existing `task22b`/`bbs_painted_triangle` mesh-test failures
reproduce on the RED parent without these changes and are unrelated.

## Deferred (non-negotiable)

Mixed-type element arrays (e.g. `["42", 42]`): upstream `parse_str_arr`
returns false, logs `invalid json array for <key>`, and **`break`s the whole
JSON key loop — every remaining project_settings key is silently skipped and
`load_from_json` still returns success** (`Config.cpp:994-998`). That
break-and-skip-of-remaining-keys behavior is **not ported by this chain**;
Ares keeps rejecting malformed arrays at the same boundary. Recorded as
deferred pending a separate decision, because porting it would change behavior
for every key following a malformed entry, not just the malformed one.

Additional deferred edges of the same upstream branch (all out of the
selected witness scope):

- Plain-string lenience: upstream's istringstream semantics also accept a
  comma-suffixed plain string (`"42,77"`) on numeric scalar options. Ares
  applies the per-type split only to array-origin wires; plain strings keep
  today's strict parse.
- Enum/string array export substitution: an array on an enum option
  substitutes the option default upstream; the embedded raw keeps the
  joined list for those kinds (validation does not read them; no witness
  exercises them).
- `coBool` arrays whose first segment is not `"1"`/`"0"` substitute via
  `enum_looks_like_true_value` upstream; Ares's raw-phase substitution
  covers the scalar-bool wire shapes the existing tests exercise, and the
  typed path errors on uncovered shapes.
- Arrays on keys with no registry definition are joined and dropped
  upstream; Ares keeps the raw entry (pre-existing raw policy for unknown
  keys).

## Established evidence, not committed Native output

All Native/API/CLI streams and comparison artifacts live under the external
`ARES_PARITY_ARTIFACT_ROOT`; no Native G-code snapshot is checked in. The
RED registers the complete-stream comparison only and pins no mismatch
location.
