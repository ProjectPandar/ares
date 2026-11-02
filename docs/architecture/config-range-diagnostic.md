# PrintConfig generic numeric-range diagnostic — source rewrite proposal

Status: proposal only; approval required before implementation.
Base: `5910df8eb0c2a9cab5958a1dcce7fdb34d075ecb`.
Upstream: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Owning upstream boundary

- `PrintConfig.cpp::DynamicPrintConfig::validate:10706-10723` applies the
  dynamic config to `FullPrintConfig`, then dispatches to `Slic3r::validate`.
- `PrintConfig.cpp::Slic3r::validate:12444` owns the numeric-range loop at
  `:12647-12692`. After the existing type-specific validity checks, and only
  if no earlier error exists for the key, `:12686-12689` inserts:

  ```cpp
  Slic3r::format("%1% not in range [%2%, %3%]",
                 opt->serialize(), optdef->min, optdef->max)
  ```

- `format.hpp::Slic3r::format` constructs `boost::format` and forwards the
  arguments without a fixed-decimal precision override. Default stream-style
  numeric rendering of the endpoints and the literal comma-space separator
  are the source behavior to port, not a new validation policy.

## Existing real public witness

The retained real Native comparison in
`cli_exit_channels::cli_exit_numeric_range_family_matches_native`, using
`tests/parity/validation/cli-exit/range.3mf`, reaches the same early invalid-value
family on Native and Ares but fails the exact stderr comparison:

- Native: `retraction_distances_when_cut: 30 not in range [10, 18]`
- Ares: `retraction_distances_when_cut: 30 not in range [10.000000,18.000000]`

This is a known baseline failure in both official/seeded inventories, not a
regression attributed to an unrelated slice. Native output is real retained
runtime evidence, not a proposed hardcoded test expectation. Implementation
qualification must repeat the unchanged public test with the pinned Native
supply and retain its actual full channels/report again.

## Exact Rust destination / scaffold replacement

`crates/ares-core/src/options/validation/loaded.rs::collect_range_errors`,
currently around `:237-243`, already collects the key-ordered errors and keeps
first-error precedence with lazy `entry(...).or_insert_with(...)`. It currently
renders endpoints with an Ares-specific fixed-six-decimal/no-space format.
Replace **only that rendering clause** with the pinned source's endpoint
format and punctuation; remove the old rendering, no flag or fallback.

Reuse the already imported `crate::options::format_number`, implemented in
`options/config_types/scalar.rs:319-345`, for the existing six-significant-digit
stream-style number spelling, subject to the real Native witness. Do not
introduce another formatter, route through G-code emission, or rewrite that
shared helper without evidence and separate scope review. Keep formatting lazy
inside the existing first-error insertion.

The remaining collector and `serialize_value` stay the current compatibility
shell around the cited validation/ConfigOption concepts. This proposal does
not claim complete `Slic3r::validate` or whole-option serialization parity.
Core remains pure and usable on browser WASM, Windows, macOS and Linux; no
filesystem, terminal, native locale mutation or UI behavior enters it. The
existing CLI error adapter remains a consumer, not a new report owner.

## Included / deferred

Included:

- literal range-message endpoint spelling and `", "` punctuation;
- preserve the existing serialized offending value, selected bounds and
  first-error/key ordering;
- unchanged public CLI exit/stdout/stderr/complete-result comparison as the
  behavioral qualification seam.

Deferred, and not silently redesigned:

- validity predicates, epsilon/NaN/nil handling, bounds/default metadata,
  normalized/dynamic override membership and unsupported option types;
- full `ConfigOption::serialize` fidelity for other value shapes;
- other specialized validation messages, bridge/width/mixed-filament rules;
- shared numeric formatter refactoring, GUI localization, CLI policy changes,
  new report channels, CI/cache changes or another slicing pipeline.

If a required actual Native witness exposes a missing adjacent formatter or
serialization prerequisite, stop and review that source boundary; do not add a
special-case key, expected-string table or approximate numeric formatter.

## Default-double stream equivalence qualification

`Slic3r::format` creates a fresh Boost format and forwards ordinary doubles.
Boost.Format's documented default stream state has precision6 and general
rather than fixed floating-point mode. Its source makes that concrete:
`boost/format/internals.hpp::stream_format_state::reset` sets precision6 with
no floatfield flag; `parsing.hpp` returns for `%N%` without changing that
state; `feed_args.hpp::put_last` inserts the argument with `os << x`.
Read-only Boost1.86 headers from the reference build establish this source
contract; they do not identify the official AppImage's exact Boost version.

Pinned `Config.hpp:837-842::ConfigOptionFloat::serialize` likewise creates a
fresh `std::ostringstream` and inserts the double. `Config.cpp:1564-1570`
writes that serialized scalar string to the public settings export.
`PrintConfig.cpp:4605-4611` defines `time_cost` as coFloat with minimum0.
The existing public Rust encoder reaches `OrcaFloat::serialize:12-21`, which
calls the exact `format_number` helper under review.

Before the endpoint port, qualify finite integral/fractional, large/small,
precision and notation-boundary samples using the real Native
`--export-settings` and public `encode_config_value_by_key`. Keep actual
Native strings as the expected values; no private formatter call, isolated
C++ kernel, copied expected algorithm or string table. Persist all Native
inputs/producer/settings and Rust comparison records. A mismatch stops the
port for review; shared formatter changes are not silently pulled into scope.
This establishes representative default-double stream equivalence, not a
claim about every f64 bit pattern or unrelated localization behavior.

## Proposed execution after approval

1. Freeze the original public range test/input and reproduce its actual Native
   RED on the approved base. No private tests, copied expected algorithm or
   committed computed golden. Do not change ROOT or the original comparator.
2. Port only the source-owned rendering clause and qualify that same full
   public comparison GREEN. No Native rebuild, error-code/tolerance change,
   skip or absent-Native fallback.
3. Retain the neighboring CLI-channel controls and compare baseline statuses;
   report any independent baseline failures rather than count them GREEN.
4. Run fmt and workspace/all-target/all-feature clippy `-D warnings`; keep
   touched source files below400LOC and submit GREEN evidence for review.

This is a bounded `PrintConfig.cpp` rewrite slice replacing a specific
non-source-literal scaffold behavior, not a self-designed Ares pipeline goal.
No production/test change is authorized by this proposal itself.
