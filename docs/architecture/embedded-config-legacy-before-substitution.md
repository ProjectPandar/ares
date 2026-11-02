# Embedded configuration: legacy before substitution

Port OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`'s
`ConfigBase::set_deserialize_nothrow` → `handle_legacy` →
`set_deserialize_raw` order. The existing embedded loader substitutes
legacy enum tokens too early; fix that ordering, not the writer or inputs.

## Exact upstream and Rust boundary

At this pinned beta, `Config.cpp:596-617` calls `handle_legacy` at602
before `set_deserialize_raw`; the failed enum/bool substitution is at
680-695. `PrintConfig.cpp:9886-9891` maps arc-fitting `1`/`true` to
`enabled`. Earlier notes' Config577-579/654-676 anchors describe the same
call chain but are not exact line locations in this beta snapshot.

Destination: `crates/ares-core/src/project/load.rs`, embedded 3MF settings
inside `load_project_with_preparation`. After rejecting JSON scalar types
upstream does not load, run the existing `ProjectSettingsRaw::handle_legacy`
before `normalize_embedded_wire_substitutions`. Do not duplicate the
existing token135/typed-rule103 migration or special-case arc mode.

The later legacy pass remains for nonempty caller overrides, after their
last-write-wins application and vector-shape normalization. Overrides are
not subjected to embedded EnableSilent substitution. An ordinary embedded
load therefore has one legacy pass, not two. Existing scalar-array binding,
preparation capture and typed-build boundaries otherwise stay unchanged.

## Load-path sharing and exclusions

- Public `load_project`, `load_project_with`, validated 3MF slicing and
  ordinary CLI 3MF slicing share this embedded path.
- A 3MF with selected profiles/`--load-settings` first calls this loader
  with empty overrides (`project/domain/profile_input.rs::from_3mf_with_profiles`),
  then overlays profile values and user edits. Its embedded base changes;
  its later merged-profile legacy/typed passes are not rewritten.
- STL plus `--load-settings` uses `profile_input/stl.rs`, not the 3MF
  embedded loader. Preset fragments/catalog composition use
  `profiles/catalog/{store,compose}.rs`, typed legacy handling and
  `options/json_deserialize.rs::deserialize_preset_config`. They do not
  share this embedded EnableSilent pass; no preset source is edited.
- Auxiliary upward-machine/flush preparation raw projections also call
  normalization helpers, but are not changed by moving this load-path
  call. Their shared helper implementations stay untouched.

This is a Config/PrintConfig rewrite slice, not an Ares normalization
policy, new loader abstraction, enum fallback or independent pipeline.
Spiral math, shared writer formatting, acceleration, FillBase and cooling
are separate slices. No filesystem or platform-specific behavior enters core.

## RED, configuration GREEN and breadth

The three immutable enabled-spiral inputs already prove Native `enabled`
versus API/CLI `disabled`. Add three direct `load_project` integration
witnesses using those exact fixture bytes and public typed settings.
They must fail before the production change and pass afterward. Their
expected enum is independently established by the retained Native exports
and the upstream legacy rule, not a rewritten input or private helper test.

Configuration GREEN means that public load-level behavior is corrected.
It does not mean the complete spiral output is GREEN: the unchanged
strict runtime-oracle cases retain every M73/geometry/footer failure.

Before the production edit, freeze config/option-coverage/profile and a
broad additional test filter with exact selected names, flags, inputs,
source identities and runtime budget in the external evidence bundle.
Run identical inventories before/after, report every status flip and
require zero prior-PASS to FAIL. Distinguish test statuses, guard returns
and actual Native parity; do not claim skipped reference guards as parity.
Keep failures and original receipts immutable. No retries, input variants,
generator normalization beyond the existing line, or weakened comparator.

Publish a separate configuration RR with public RED/GREEN evidence,
breadth transitions, fmt/strict Clippy and explicit remaining full-output
failures. Stack the separately authorized spiral implementation afterward.

## Qualified configuration result

The unchanged three public load witnesses moved from `Disabled` failures
at `15acc53be7d9ce03c1256ede9c108b0f30db7a82` to three `Enabled` passes.
The only production delta is the embedded/override ordering in `load.rs`;
legacy rules, preset decoding, writer code and strict comparators are unchanged.

The frozen 666-test inventory completed once before and once afterward:
343 passed, 323 failed and zero ignored in each phase. There were **zero
status flips and zero prior-PASS to FAIL**. These are test-status counts,
not a claim that guard-returning or non-parity tests establish Native parity.
Each phase retained858 Native command-status records; the same ordinary
producer, reference flags, registered beta profile resources and filter were used.

The three original enabled-spiral cases now have `enabled` in Native,
API and CLI, stable Native replays and raw API/CLI equality. Their complete
outputs still fail with M73 retained; the old G17/linearized mismatch is
absent from the first remaining external non-M73 difference. This is
configuration GREEN, not spiral/FillBase/FULL closure. All323 retained
failures, exact execution trees, input/output identities, unchanged test
postimages and lint receipts are in the external evidence bundle. This
result note is a post-execution documentation-only delta, not a rerun.
