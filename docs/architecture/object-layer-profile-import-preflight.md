# Stored layer profile import preflight and public RED

Base `c22f5a81d`; approved boundary: [P/R spec](object-layer-profile-and-range-slicing-spec.md).
This is Slice P only. No production change or missing-height range admission.

## Import observations

Official and seeded each executed all eleven authored input variants on CPU11,
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

## Evidence

- Diagnostic capture with per-case cwd/log isolation:
  `/tmp/worker6-profile-import-preflight-qx25mdjz/{inputs.json,results.json,diagnostics.json}`.
- Earlier CPU11 selection capture:
  `/tmp/worker6-profile-import-preflight-mp8ehtc9`; complete outcomes retained,
  but log isolation was missing, so it is not the diagnostic authority.
- Initial CPU7 capture `/tmp/worker6-profile-import-preflight-336_c7_a` predates
  worker5's core-ownership notice; excluded from formal evidence. All later
  Native invocations use CPU11.
- Reproduction script `/tmp/worker6-profile-import-preflight.py`; no Native
  producer was rebuilt, no processor oracle was used, and no full was run.

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

Evidence `/tmp/worker6-profile-red.zWjapr/{official,seeded}`: each exit100,
0 PASS/2 FAIL. Cache receipts, original input and Native bytes are retained
under those external artifact roots. No captured Native file is committed.
