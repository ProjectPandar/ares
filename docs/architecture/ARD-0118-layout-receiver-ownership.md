# Settings layout receiver ownership

## Upstream boundary

OrcaSlicer `40ecc66f7`, `src/slic3r/GUI/Tab.cpp`, owns settings page,
option-group and row ordering. In `TabPrinter::build_fff`, lines 5514–5546
populate `optgroup` (Advanced), lines 5550–5561 create and populate `belt_og`
(Belt printer), and lines 5568–5573 create and populate the scoped `mf`
(Machine frame transforms). Lines 5576–5621 resume adding rows to Advanced;
lines 5623–5624 create Plugin Configuration.

The destination is `web/scripts/extract-process-layout.mjs` and its generated
`web/app/src/sidebar/{process,preset}Layout.ts` data. These tables represent
upstream GUI ordering, not independently designed slicing behavior. The Rust
option and slicing boundaries remain unchanged.

## Included behavior

Track each option-group assignment by its receiver variable. Append rows to
the group named by the append call rather than the most recently created
group. Preserve page and group creation order, option order within each group,
labels, indices and widget metadata. Extract every tab from the same source
revision and compare all generated rows, groups and pages. Publish only the
printer/filament table here; the process table refresh belongs with the typed
configuration fields needed to display its additional release options.

## Deferred behavior

Typed belt, frame-transform and plugin options are a separate PrintConfig
rewrite slice. Belt slicing, G-code transforms, conditional option visibility,
general C++ parsing, and the process table refresh are not included. Existing
typed option structures are unchanged temporary representations of their
upstream configuration boundary.

## Failure modes and behavioral verification

A scoped group can divert later rows from an earlier receiver. Reassigned
receiver variables can retain obsolete groups. Literal rows, option-variable
rows, multi-option lines, separators, widgets and loop-generated rows can each
lose their owner. Group creation order can accidentally become append order.

Exercise the generated settings layout through its public table exports.
Assert that the six Advanced rows follow `disable_m73`, that belt and machine
frame rows remain in their own groups, and that group order matches Tab.cpp.
Run the complete settings browser scenario before and after regeneration;
compare process, filament and printer tables to enumerate every change. Check
that regeneration is repeatable. Existing G-code parity scenarios must remain
unchanged because this slice does not modify configuration or slicing code.
