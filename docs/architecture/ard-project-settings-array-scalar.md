# Project-settings array-valued scalar — approved Rust rewrite slice

Chain: `w1/supportcubic40-create-offset-line` on RED parent
`1f784dbae58dd1f27c51c688e68176a85623dbe9` (main=aecea85be lineage).
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Status: COORD approved chained RED→GREEN after the sealed read-only PINPOINT
(`support-pinpoint-array-scalar-aecea85be`, seal `2b400a5f...a943c2`).

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

## Deferred (non-negotiable)

Mixed-type element arrays (e.g. `["42", 42]`): upstream `parse_str_arr`
returns false, logs `invalid json array for <key>`, and **`break`s the whole
JSON key loop — every remaining project_settings key is silently skipped and
`load_from_json` still returns success** (`Config.cpp:994-998`). That
break-and-skip-of-remaining-keys behavior is **not ported by this chain**;
Ares keeps rejecting malformed arrays at the same boundary. Recorded as
deferred pending a COORD decision, because porting it would change behavior
for every key following a malformed entry, not just the malformed one.

## Established evidence, not committed Native output

All Native/API/CLI streams and comparison artifacts live under the external
`ARES_PARITY_ARTIFACT_ROOT`; no Native G-code snapshot is checked in. The
RED registers the complete-stream comparison only and pins no mismatch
location.
