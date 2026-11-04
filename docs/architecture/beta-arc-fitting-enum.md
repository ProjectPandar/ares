# Beta arc-fitting enum rewrite boundary

Source revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
Destination base: `54b2fff56c814e65eb480dd965d6f782e96a0375`.

## Non-negotiable scope

Port `PrintConfig.hpp:238-242,1706`, `PrintConfig.cpp:256-261,4426-4444`:
`enable_arc_fitting` is ArcFittingMode, not Bool. Exact enum vocabulary is
`disabled`, `enabled`, `sparse_infill`, default Disabled, codes0/1/2.
The source metadata labels are Disabled, Enabled and Sparse infill only.

Upstream `handle_legacy` at PrintConfig.cpp9886-9891 explicitly converts
1/case-insensitive true -> enabled and0/case-insensitive false -> disabled.
Port that conversion only where the existing token/typed legacy paths mirror
handle_legacy; do not invent Boolean truthiness or additional spellings.

Rust destination: typed process GCodeConfig field, public config definition and
metadata, public decode/encode, config export and the requested/effective test
adapter. The old OrcaBool field is replaced, not retained as a second source of
truth or an engine-version fallback. Existing metadata records outside this
key remain unchanged. No other scalar/vector option migration belongs here.

## Public Native-first RED

The agreed seam is runtime Native `--export-settings` on the unchanged Ender3
input -> public `decode_config_value_by_key`, `ProcessGCodeSourceOptions`
deserialize/serialize and `encode_config_value_by_key`. Native produces the
canonical disabled token from the historical input; the old Boolean decoder
rejects it. The test observes public values, not private implementation fields.
Record Native identity/input/output and Ares exported JSON externally.

## Existing consumers cannot silently collapse sparse-only into true

The source-derived mode predicates are in scope solely to replace existing
Boolean gates, without changing the fitting algorithms:
- LayerRegion.cpp1102-1114: only Enabled, or SparseInfill with
  erInternalInfill, fits a path; spiral mode disables fitting.
- Layer.cpp383-419: support/raft fitting uses Enabled only.
- PerimeterGenerator.cpp1531,2677: fine perimeter simplification uses
  Enabled only. Sparse-only is not an all-wall mode.
- GCode.cpp9915: emission replays fitted results unless mode Disabled;
  GCodeWriter.cpp1218 similarly linearizes spiral lift only for Disabled.
- WipeTower/WipeTower2 constructors use Enabled only; current tower behavior
  outside the existing Ares port is not newly qualified by metadata.

Any existing earlier fill simplification gate must be checked against the cited
beta source rather than preserving a stale 2.4.2 Boolean shortcut. Active
arc-algorithm parity, other beta planner changes and unrelated residuals remain
unqualified unless closed by the unchanged affected witness subset.

## Before/after accounting

Use the accepted dual-mode baseline results as the before authority. Select only failed
cases whose captured failure mentions the arc Boolean decode or its
requested/effective mismatch. Map to surviving/new test names explicitly;
record deleted cases rather than resurrecting them. After GREEN run only that
affected subset in both modes, reporting pass/fail and remaining failure
boundaries, plus the new RED. No full parity run or comparator/domain relaxation.
A downstream failure after eliminating an arc blocker is progress, not a
manufactured PASS or complete feature qualification.
