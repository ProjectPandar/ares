# ARD: Preserve CLI loaded-system group metadata semantics

Status: proposed. [SPEC/fixture/budget](perimeter-input-dispatch-spec.md).
Seam v2 NO-RED closure remains accepted. No production edit in this slice.

## Source owner / destination

Orca8500fcdcc PresetBundle.cpp::construct_full_config69-236 owns the positional
inherits_group concept. CLI::run source-specific loaded rules clear system
machine2838-2842/process2981-2985 and retain filament parent only for User/user
2189-2193, assigning3161-3163. update_full_config2750-2812 excludes raw inherits;
GCode.cpp::append_full_config5591-5648 preserves resulting metadata in full output.

Rust destination: CLI loaded-source metadata preparation compose_load_settings
69-124, existing group constructor composition/metadata23-32/compose87-91 and
profile_input/overlay247-273 as needed. Group currently collects raw inherits
without from-system distinction. Existing group/raw overlay is a compatibility
shell for upstream preset/CLI concepts, not an Ares-specific inheritance policy.
Do not blanket-change general bundle/catalog semantics to match the CLI route.

## Non-negotiable decisions

- Identical input includes model AND loaded JSONs. Bare3MF-versusloadedNative
  comparisons were harness mismatch, not the five-key config-resolution bug.
- One same-request Ares API+CLI check now matches all five keys and
  raw API/CLI output. Only actual inherits_group remains different: Native empty
  three slots versus Ares system parent names. Preserve this metadata REDcandidate;
  do not suppress footer to manufacture NO-RED.
- System-source identity, not merely raw inherits presence, determines CLI group
  values. User/user and unloaded slots must retain upstream policy and order.
  Do not clear every group, erase inherits fields or mutate positive fixture values.
- Effective-config divergence under identical request remains observable RED;
  do not require equal actual config before RED. Whole ordered output must pass,
  retaining all motion/arcs/M73/statistics/footer under generator-only normalization.
- Separate SPEC/fixture/budget ->fresh public test-only RED ->
  production. External retained-Native witness is not a fresh
  paired/seeded RED test. No retries or added cases follow automatically.
- General vendor inheritance, defaults, compatibility metadata, seam geometry,
  dispatch/order/brim/travel are deferred. No new resolver/API, fallback, guard,
  filesystem/terminal I/O in core or broad cleanup. All Tier1 platforms unchanged.

## Trade-off

Shared catalog composition is reused, but bundle group semantics are not the
same as loaded CLI system-file semantics. The minimum correction must carry
source status through the existing CLI metadata handoff, without inventing a
second settings pipeline or weakening complete G-code acceptance.
