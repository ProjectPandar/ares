# SPEC: BBL M205 Z/E tail defaultfloat precision

Base: `2e848a33e713493c9b78e724b0d107f31aace3d8`.
The coordinator approved this specification and requires separate review requests
for the RED and GREEN stages. No FULL run is authorized.

## Source owner and architectural constraints

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:298-299`, in `set_jerk_xy`, appends the
BBL Z/E cache values using `std::setprecision(2)` on the default stream. This is
ostream defaultfloat, equivalent to printf `%.2g`: two significant digits,
a retained leading zero for subunit values, and fixed/scientific notation chosen
from the rounded exponent. It is not fixed-two or axis-three formatting.

The Rust destination is the existing BBL suffix in
`gcode_emit/motion/jerk.rs:37-43`, which currently formats `max_jerk_z` and
`max_jerk_e` with `format::axis`. Change only those suffix serializers; preserve
the numeric caches, raw headers, XY fields and command admission.

The existing `motion/format.rs::ostream` owns the six-significant-digit `%g`
rewrite. Parameterize that implementation's significant-digit count. Preserve
all existing `ostream(value)` callers and their six-digit behavior. Because Rust
has no default function arguments, the existing name may remain a thin wrapper
that delegates to the single parameterized implementation. Do not duplicate the
`%g` algorithm or maintain a second formatter. The BBL suffix must request two
significant digits through that same implementation.

Retain round-first exponent selection, sign handling, scientific exponent syntax
and trailing-zero removal. Do not merely prepend a zero. Preserve the existing
zero behavior and default-six formatting; unrelated signed-zero changes are out
of scope.

Included after RED review: shared formatter precision plumbing and the two BBL
Z/E consumers selecting precision two. Deferred: BBL/flavor gate placement, raw
headers and configuration, caches, tolerance, clamps, XY serialization, other
writer fields, and a general formatter redesign. W1's `auto_lift.rs`, M73/time
estimator and W3's tokenizer, fan and zero-E files are explicitly excluded.
No output masks, processor compensation or lint suppressions are permitted.

## Clean source-derived witness

Use the existing GUI-generated BBL project
`tests/parity/bbl-layer-object-labels/one-object.3mf`, with the tower disabled and
`default_jerk=9` in every case. The target changes only `machine_max_jerk_z` to
0.45. Controls retain the stock Z value or set it to 2.5. These are real saved
machine-jerk settings; no Type1 tower behavior or fan/tokenizer work is involved.
No claim is made about GUI decimal display precision or recommended tuning.

The target's first normalized difference is byte 67333: Native emits
`M205 X9 Y9 Z0.45 E2.5`, while Ares emits `M205 X9 Y9 Z.45 E2.5`. All 299 changed
lines contain only that missing leading zero in M205 commands. Line counts and
all M73 records are identical. The acceptance comparator remains unmasked.
The stock and Z=2.5 controls match over their complete streams.

The initial Z=0.4567 discovery case exposed both rounding and M73 placement
changes. It is excluded from this RED and is not a request for time-estimator
work. Do not commit that input archive. The formatter still needs correct
significant-digit semantics, not a patch limited to the clean witness's zero.

Input SHA256 values:
- Stock control: `8218407a4f814505bf6df4f4c82b0c67a39c73acfaf938da72ba5144f50fe32f`
- Z=0.45 target: `5eb57eb85313786b8b3f19cf8591771c2d79aeda963e571c619a20ba2fb18a52`
- Z=2.5 control: `70db0f1afd38380c28f93b2d435b01120d49cb74a46cf3925c10ed6ddcf03afe`

Discovery evidence is in `/home/indexyz/ares-evidence/w4-bbl-jerk-tail-precision/`,
including `SPEC-PROPOSAL.md` and `isolated-diff-audit.json`. Eight fresh Native
slices across four discovery inputs were replay-stable on CPU17. Discovery used
the exact base-tip CLI; it does not qualify public API, seeded, RED or FULL tests.

## RED gate, failure modes and review boundaries

Freeze only the three authorized archives and add direct calls to the existing
`native_stream_case` seam. First qualify fresh Native replay. Then use a Native
callback to inspect the BBL M205 Z/E text against the saved input values, without
hardcoded Native coordinates, counts or reference output goldens. Finally compare
complete Native/API/CLI output in both modes, including timing, headers,
configuration and footers. Callback observations are not comparison filters.
Native outputs must remain external artifacts.

Expected RED: the target fails; both controls pass; Native replay, callbacks and
raw API/CLI equality qualify independently. Commit RED, send a review request,
and hold before production changes.

Failure modes to consider before implementation: fixed axis precision instead of
defaultfloat, lost leading zero, incorrect two-digit rounding or exponent choice,
changes to existing six-digit callers, duplicated algorithms, and accidental
header/XY/cache changes or M73 masking. The clean public witness alone does not
prove all rounding and exponent corners. Review the implementation and run
existing formatter-neighbor tests; do not add unit tests after production code
or expand the public Z=0.4567 case beyond the approved RED scope.

After RED approval, implement only the shared precision parameter and BBL callers.
Keep RED inputs and tests unchanged. On CPU17 with PIN17, one build job and one
parity thread, run fmt, workspace all-target clippy, the same three directed cases
in official then seeded mode, and appropriate existing formatter neighbors.
Send a GREEN review request and hold. No FULL, merge or push. Preserve normal
registration separators, and keep every touched Rust file at or below 400 lines.
