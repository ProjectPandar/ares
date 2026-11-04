# Native probe: release spiral range branches

Upstream `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
See [the amended spec](spiral-single-range-spec.md).

## Clarification and conclusion

The initial conservative note treated the multi-range debug assertion as a
rejection boundary. That interpretation is superseded: official2.4.2 is NDEBUG, so `assert(!spiral_mode)` is compiled out. Its
multi-range body executes defined ordinary ranged slicing. Deterministic
completed R1-like release runs are therefore literal-port authority.

All18 serial Native invocations completed with exit0, CONFIG_BLOCK_END and
spiral_mode1. Each case is byte-identical across three fresh runs within each
mode after removing ONLY the generated-by timestamp suffix. No retries,
producer rebuild, Native source patch or widened normalization was used.

## Witnesses and measured results

Source input is `tests/parity/spiral-hole-projected-split/spiral.3mf`,
SHA256 `7737bc4991c554b645ceed6fe819462cfb1ece3ad2b6cfdf9095dacd78dbde27`.
C0 is that exact archive. R1/R2 preserve every original member payload and
append ONLY `Metadata/layer_config_ranges.xml`, object ordinal1 (not resource2).
Both have explicit layer_height0.2 plus non-default outer_wall_speed17.

- R1 range `[0,1000]` covers the entire model but still produces a default tail.
- R2 range `[0,0.00005]` is ignored by upstream EPSILON0.0001, leaving the
  default range. It is an admission/no-op witness, not active-speed evidence.

| Case | GDB-proven branch/count | Official | Seeded |
|---|---|---|---|
| C0 | single, 1 | 51 layers / 3 PASS | 51 layers / 3 PASS |
| R1 | multi, 2 | 50 layers / 3 PASS | 50 layers / 3 PASS |
| R2 | single, 1 | 51 layers / 3 PASS | 51 layers / 3 PASS |

R2 complete normalized G-code equals C0 byte-for-byte in BOTH modes.
R1 differs from C0 and has50 rather than51 physical layers. In BOTH modes
it contains102 `F1020` commands (17mm/s), versus zero in C0/R2, confirming
that its non-default speed override is active. The positive XY+Z+E move metric
is1 in R1 versus236 in C0/R2: preserve that actual release behavior rather
than forcing helical output solely because spiral_mode remains1.
Preserve R1's Native-derived profile/Z schedule and speed/motion in strict
comparisons; do not force its count to the control or attribute every
difference solely to speed (the input also authors an explicit height range).

### Frozen probe input SHA256

- C0: `7737bc4991c554b645ceed6fe819462cfb1ece3ad2b6cfdf9095dacd78dbde27`
- R1: `00cf2cd954a57cc4be34bb5eeca90b41fbe92cf017e75fcf51f3f29d0d116133`
- R2: `32d081707888f472bd00f127a78e56514cb0351bf5c9ba480f08c9ac217ba568`

### Normalized Native output SHA256 (three identical per cell)

- official/C0: `09069809472deccc940f38ae4aa475041b253af1e438d6879a9565806996b739`
- official/R1: `70b26fa9a72633af852655c250da6253481a5e255e0328a6c8c8b243ce559dc3`
- official/R2: `09069809472deccc940f38ae4aa475041b253af1e438d6879a9565806996b739`
- seeded/C0: `09069809472deccc940f38ae4aa475041b253af1e438d6879a9565806996b739`
- seeded/R1: `70b26fa9a72633af852655c250da6253481a5e255e0328a6c8c8b243ce559dc3`
- seeded/R2: `09069809472deccc940f38ae4aa475041b253af1e438d6879a9565806996b739`

## Seeded read-only GDB confirmation

Pinned ELF matches the existing seed-replay producer and matching separate
debug info. Observers read parameters only; no inferior calls/config writes.

- C0 and R2: global vector count1, `[0, DBL_MAX]`, null config,
  one volume-region record; actual mode PositiveLargestContour, mode_below
  Regular, normal_below_layer3.
- R1: global vector count2: `[0,1000]` with config/one volume-region record,
  plus `[1000, DBL_MAX]` with null config/zero volume-region records.
  Actual mode Regular, mode_below Regular, normal_below_layer0. Thus an empty
  tail still selects the release multi-range path, even with only one occupied
  interval and one distinct effective configuration.

Optimized line-table caveat: a breakpoint requested at source line221 also
maps shared machine code in single-range runs. The observer's source-line
label is NOT a branch predicate. The conclusion uses independently read
actual global count, mode and cutoff; those are unambiguous. Raw receipts
preserve the source-line labels rather than silently rewriting them.
The initial observer attempt did not reload symbols after setting the debug
directory and had no samples; it is retained as non-evidence. Only verified
observers with real parameter samples and complete output qualify this table.
Official execution is direct, not debug-observed; source pin plus official
replays and the matching seeded branch observation form the stated evidence.

## Literal port proposal (not implemented)

1. Replace guard2's blanket rejection with support for Native-completing,
   deterministic C0/R1/R2-like inputs. They are witnesses, not a shape whitelist.
   Preserve unrelated guards; do not invent rejection based on the removed
   debug assertion, or silently accept Native-rejected/nondeterministic cases.
2. Preserve the original global LayerRanges assignment, including height-only
   entries, default gaps/tails, EPSILON and overlap precedence. Correct the
   prior height-profile boundary assumption that configuration deduplication
   collapses this vector. Retire the temporary height-only candidate filter
   through one source-owned assignment, not an independent second planner.
3. For actual count1, reuse the matching-volume/region source branch and existing
   bottom threshold plus upper PositiveLargestContour path.
4. For count>1, use base slicing mode across the intersections and the source
   per-volume interval filter/scatter. Do not panic in model_part_threshold or
   substitute single-range mode just because one interval intersects the model.
5. Write Native-first public API/CLI RED for identical C0/R1/R2 in BOTH modes;
   retain whole ordered-byte checks with existing generator-only normalization,
   plus actual physical layers/Z/spiral motion. Include height-only/mixed-range
   regression controls for the assignment refactor. No normalization/tolerance
   changes, assertion waivers, scalar defaults or new public diagnostic APIs.
6. GREEN: minimal existing-owner changes,
   both-mode directed strict parity, required-oracle non-parity and fmt/clippy.
   Inherited failures must be named, not relabeled as all green.

## Evidence outside the repository

The original conservative R1 classification is historical and superseded by
the release assessment above; its input hashes and measured outputs are
unchanged. No Native output is committed.
