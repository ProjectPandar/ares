# Stored layer profile import preflight and public RED

Boundary: [P/R spec](object-layer-profile-and-range-slicing-spec.md).
This is Slice P only. No production change or missing-height range admission.

## Import observations

Official and seeded each executed all eleven authored input variants
sequentially, with independent fresh directories. Every invocation returned 0.
The two producers' generator-filtered complete outputs agree for every input.

| Input | Native selected data | Import diagnostic |
|---|---|---|
| Absent / empty profile member | Ordinary layer schedule | None |
| Four values / odd value count | Ordinary layer schedule | `Found invalid layer heights profile` |
| Valid six-value profile | Stored profile schedule | None |
| Duplicate ordinal, different second profile | First profile retained | `Found duplicated layer heights profile` |
| Unknown ordinal | No object receives the stored data | None |
| Malformed record without separator | Ordinary layer schedule | `Error while reading object data` |
| Nonnumeric first Z token | `std::atof` converts it to 0; valid profile schedule | None |
| Top mismatch / first-height mismatch | Ordinary layer schedule regenerated | None |

Ordinary Z/HEIGHT pairs are (0.3,0.3), (0.5,0.2), (0.7,0.2).
The valid stepped profile produces (0.3,0.3), (0.45,0.15), (0.6,0.15).
These are observed selection outcomes, not new hard-coded Native authority.

`bbs_3mf.cpp:2840-2881` collects record errors and continues importing; the
unknown ordinal is not associated at :2087-2090. Its `std::atof` at :2878 is
permissive, unlike a strict Rust number parse. `PrintObject.cpp:3818-3828`
regenerates profiles with unsuitable top or first-layer height.

`_BBS_3MF_Base` owns `m_errors`/`add_error`/`log_errors`
(`bbs_3mf.cpp:726-739`); `load_bbs_3mf` logs these at :8962 even on success.
Default CLI stdout/stderr and result warnings are empty here; the messages
are in its default `00000.log`, not an import failure or dropped feature.
Ares must not replace these accepted inputs with InvalidInput or Unsupported.
Any diagnostic representation must preserve successful slicing and record-level
selection, remain portable, and avoid filesystem logging in ares-core.

## Diagnostic ruling

A diagnostic getter is rejected: `_BBS_3MF_Base::m_errors`
only feeds BOOST_LOG; it changes neither G-code nor slicing/CLI success.
No public diagnostic API or unused error carrier is added. The importer follows
upstream record selection and comments each nonfatal skip with the exact source
lines. This is source-defined import behavior, not a silent legacy fallback.

## Capture method

Diagnostics were captured with per-case cwd/log isolation. No Native producer
was rebuilt and no processor oracle was used.

## Public RED

`runner::object_layer_profiles` compares absent input first against runtime
Native through the unchanged complete-byte comparator. Then ramp/step inputs
must have distinct Native schedules and match the public validated byte API,
including all layer Z/HEIGHT rows and every ordered output byte.

Both official and seeded execute two tests and fail at the existing
`UnsupportedProjectFeature("layer_height_profile")`; each absent-input
complete-output control passes before that failure. The production source is
unchanged. This is a supported Native input rejected by Ares, not a compile
failure or a binary64 construction substituted for a Native oracle.

Each mode exits 100 with 0 PASS/2 FAIL. No captured Native file is committed.

## Additional control corrections and deferred finding

After the original RED, broader controls were added. Native-to-Native import
controls now apply the existing `normalize_one_generator_line(Orca)` to both
sides and assert exact bytes. The Ares-to-Native comparator is unchanged. The
initial misuse of its Ares generator check is not counted as a production failure or a weakened comparison.

The new two-object fixtures' `.rels` serializer originally emitted
`ns0:Relationship`; upstream compares literal `Relationship`
(`bbs_3mf.cpp:4830`). Only namespace spelling is corrected in that fixture
generator; geometry, placements and settings stay unchanged. Corrected fixtures
require a fresh RED reconfirmation on unchanged main production before acceptance.

Reconfirmation uses unchanged production with corrected tests/inputs only.
Each mode returns
exit100, 0 PASS/7 FAIL, exclusively at the existing profile rejection. Every
inactive full-byte control passes; no producer rejection or timeout remains.
The `crates/ares-core/src` diff is empty. Ramp/step from the
original RED are byte-identical inputs; the new `.rels` fix preserves geometry,
placements and settings.

The combined precise-Z/shrinkage control was added after the original RED, whose
provenance contains only absent/ramp/step. Its no-profile baseline has a real
first difference at line 3: Native 11 layers versus Ares 12, in both modes.
It is removed from P and frozen as an independent finding.
Targets: `Slicing.cpp:67-115`, `PrintObject.cpp:3732-3780`,
`Print.cpp:3851-3884` and `PrintApply.cpp:1534,1776`. No production fix for this
finding is included in P; a new source-cited SPEC→RED slice follows P.
