# ARD: Placeholder regex comparisons require a whole-subject match

Docs-only SPEC, then Native-first formal RED. Independent base
47821c760d5ea86102a34bbc78c092bbaf381214. No production change in this phase.

## Upstream owner and non-negotiable Rust destination

OrcaSlicer2.4.2 `libslic3r/PlaceholderParser.cpp:699–704`, client::expr::
regex_op, uses regex_match over the WHOLE subject at701;702–704 negates
that result for !~. The wrappers712–713 route =~ / !~ through this same
operation. This is upstream placeholder-expression behavior, not an
Ares scripting-language or infill-template feature.

Rust destination ONLY
`crates/ares-core/src/project_slice/gcode_emit/expression.rs:148–157`,
Parser::parse_compare's RegexMatch/RegexNotMatch branch. Current
Regex::is_match searches for any matching SUBSTRING. Future
GREEN must compile an anchored pattern **^(?: + pattern + )$**
with the existing regex crate, testing existence of a full-span match,
then retain the existing negation. No new engine/helper/public API or
string-specific case/fallback. Do not inspect only the first unanchored
match's span: a|abc on abc must still succeed, as Native does.

Non-negotiable deferrals: dialect/Unicode/escaping/error text, other
operators/functions/regex users/caching/config parsing and all rendering.
**Do not touch value.rs**: its double-rendering chain is a separate slice. No attempt
to solve that separate behavior as part of this comparison slice.
Any strict residual after future bounded GREEN requires STOP/report,
not scope expansion, output normalization or a different regex engine.

## Ordinary GUI, single-key actual-exported INPUT pair

Machine G-code / Machine start G-code text editor accepts placeholders.
Both use the same ordinary closed8-sided cylinder radius6mm/height1.2mm,
18vertices/32triangles,six layers,one object/part/filament,Classic1wall,
no infill/top/bottom/brim/skirt/support/spiral/fuzzy/scarf/stagger/wipe.
Sole changed JSON key **machine_start_gcode**, changing the regex pattern:

```
control:  ;MACHINE_START\n;W5_VALUE {"abc" =~ /abc/}\nG28\n
positive: ;MACHINE_START\n;W5_VALUE {"abc" =~ /b/}\nG28\n
```

These are valid ASCII GUI inputs, no backslash/Unicode/locale or private
option. Comment placement isolates Boolean semantics without unsafe
printer actions; the same operator governs ordinary conditional G-code.
No physical geometry/material mismatch claimed from this witness.

Actual official-exported INPUT3MFs copied unchanged:

| Input | Whole SHA256 |
|---|---|
| whole-match control |03a3282d12e670df11f7e690e1dea0b80a36958427b38e2af535bef850b7bae5|
| substring positive |a394d6dc4525cd8f7d60d7eac592ff0160843181b8a1dc6587354b569a82868e|

Identical ZIP inventory, only Metadata/project_settings.config differs,
only machine_start_gcode differs. Freeze whole/per-entry hashes and
export provenance, not expected G-code snapshots or post-export edits.

## Native-first strict feedback and constraints

Discovery official1/official2/seeded stable: control completePASS all3,
positiveONE strict difference all3. First strict non-M73 line27:
Native evaluated Boolean false / Ares true in the probe comment.
All supplied bytes compared, ONLY existing canonical generated-by
identity/date normalization. NO comment/M73/time/header/E/geometry/
config/trailing-line mask, sorting or numeric tolerance.

Cheap additional screens (not formal dual-mode regression credit):
!~ whole control passes; partial !~ reverses the Boolean mismatch;
a|abc and .*b.* on abc both pass. These constrain whole-match/negation
and prohibit a naive first-search-span or literal-equality rewrite.
They remain external screens unless a later phase explicitly adds them;
the primary two-input pair is sufficient for formal RED here.

No GDB/probe/counterfactual or whole-GREEN sufficiency claim; direct
source-call semantics and repeatable whole-output gap localize this
candidate. Earlier escape-containing exploration was rejected because
Native export altered its escape, not treated as a parity case. The
retained ASCII pair avoids that separate round-trip question.

## Scope / exclusions / formal RED gate

Included in GREEN only whole-subject RegexMatch/RegexNotMatch
comparison with anchored compile and unchanged negation. Everything else
above deferred, especially value.rs and other users' regex semantics.
No global rendering/formatting/error/default changes or new dependency.

Avoid painted/sidecut,brim/EFC,combine-solid/aligned
infill-template,concentric sparse,spiral/processor/Lightning join/
raft Contact width,zero-area/occlusion/medial/interlocking/nearest/
value rendering. This is machine G-code PlaceholderParser, not the
aligned sparse-infill template.

Two DIRECT main native_stream_case::check calls for actual exported
positive/control. Existing helper double-replays Native then delegates
complete validated API/bare CLI comparison to native_api_cli. No copied
replay/normalization/JSON/orchestration, Native output constants/assertions,
private seam, ignored/expected-failure decoration or G-code snapshot.
Input-only3MFs + input manifest/README. Official then
seeded controlPASS/positiveactualFAIL with stable repeats and complete
public attempts. Later GREEN freezes tests/inputs/helpers.
