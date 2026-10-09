# KSR runtime Native authority

## Decision

The original `ksr_fdmtest_v4.gcode` capture's historical producer is not
reproducible. It is not the authoritative Native reference for Ares output.
The independently initialized producer recorded by ARD-0068/0069/0073/0074
and `initialized-reference.json` is authoritative for this KSR input.
It remains distinct from untouched official Native behavior.

Source boundary: OrcaSlicer `8500fcdcc`, `Print.cpp:2618-2638`
`Print::export_gcode`, `GCode::do_export`, and
`GCode.cpp:5591-5643` `GCode::append_full_config`. Rust consumer boundary:
CLI complete-output E2E and migrated CONFIG_BLOCK E2E. Core retains pure Rust
configuration checks and never runs Native. No production pipeline changes.

## Preflight conclusion

Two serial fresh captures use the unchanged KSR input, seed zero, CPU zero,
and the historical initialized wrapper and ELF. Both executable SHA values
match their recorded identities, and both captures succeed.

- Raw initialized SHA differs solely because the validated generator timestamp
  changes. No other bytes change after the existing generator normalization.
- Normalized initialized whole-output bytes and hash reproduce exactly in both
  captures and agree with each other.
- Initialized CONFIG_BLOCK bytes and hash reproduce exactly in both captures:
  49065 bytes, with unchanged order, formatting, keys and values.
- Original capture's normalized whole-output and CONFIG_BLOCK hashes do not
  reproduce with that producer. First normalized difference is printing time
  on line 3; original block is 49005 bytes and first differs at block line 76.

Complete producer identity, commands, raw captures, all three hash classes and
first differences are retained outside Git in
`/tmp/worker6-ksr-p3-preflight/hash-audit.json` and its capture directories.
No Native output or generated receipt is added to the repository.

## Consumer contract

All Ares-versus-reference comparisons migrate to freshly generated initialized
Native output with the unchanged complete ordered generator-only comparator.
CONFIG_BLOCK comparisons use runtime block bytes directly, without introducing
a replacement fixed hash. Any Ares/runtime Native mismatch remains a real RED
and must be reported; no tolerance or output masking is authorized.

Assertions authenticating only a disappearing frozen file may disappear with
that file: historical raw/normalized hashes, frozen line count, receipt/output
identity, and original CONFIG_BLOCK size/hash self-checks. The deletion review
must list their exact original assertion text and individual deletion reasons.
Input SHA/package assertions and all Ares-versus-Native behavior assertions
remain. The original CONFIG_BLOCK hash is never rebound to the initialized
hash. Runtime reference integrity uses the existing producer-bound cache.

Native consumers follow shared required/optional availability policy before
cache use. Policy tests run in default environments using a temporary external
directory. Browser comparison retains actual WASM bytes, loses its old ignore
attribute, and receives explicit Native environment and artifact destination.
